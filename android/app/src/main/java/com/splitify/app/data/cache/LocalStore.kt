package com.splitify.app.data.cache

import androidx.room.withTransaction
import com.splitify.app.data.dto.CreateGroupRequest
import com.splitify.app.data.dto.DebtSnapshotDto
import com.splitify.app.data.dto.GroupDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.GroupSnapshotDto
import com.splitify.app.data.dto.GroupSummaryDto
import com.splitify.app.data.dto.RecurringDebtSnapshotDto
import com.splitify.app.data.dto.ShoppingListSnapshotDto
import com.splitify.app.data.dto.UserDto
import com.splitify.app.data.local.SplitifyDatabase
import com.splitify.app.data.local.entity.GroupSnapshotEntity
import com.splitify.app.data.local.entity.GroupsCacheEntity
import com.splitify.app.data.local.entity.IdMappingEntity
import com.splitify.app.data.local.entity.LocalMetadataEntity
import com.splitify.app.data.network.AppJson
import java.time.Instant
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Local-first store for group data.
 *
 * Snapshots are kept as JSON: they mirror the server payload exactly, so there
 * is no entity/DAO/mapper layer to keep in sync. Every mutation is committed in
 * the same Room transaction as its outbox operation, which makes optimistic UI
 * and the pending queue atomic.
 */
class LocalStore(
    private val database: SplitifyDatabase,
    private val scope: CoroutineScope,
) {

    private val snapshotDao = database.snapshotDao()
    private val outboxDao = database.outboxDao()
    private val idMappingDao = database.idMappingDao()
    private val metadataDao = database.metadataDao()

    private val _groups = MutableStateFlow<List<GroupSummaryDto>>(emptyList())
    val groups: StateFlow<List<GroupSummaryDto>> = _groups.asStateFlow()

    private val _snapshots = MutableStateFlow<Map<Long, GroupSnapshotDto>>(emptyMap())
    val snapshots: StateFlow<Map<Long, GroupSnapshotDto>> = _snapshots.asStateFlow()

    private val _idMappings = MutableStateFlow<Map<String, Long>>(emptyMap())

    private val _pendingCount = MutableStateFlow(0)
    val pendingCount: StateFlow<Int> = _pendingCount.asStateFlow()

    private val _isLoaded = MutableStateFlow(false)
    val isLoaded: StateFlow<Boolean> = _isLoaded.asStateFlow()

    private val writeMutex = Mutex()

    @Volatile
    private var mutationVersion: Long = 0

    /** Increases on every local mutation; used to avoid clobbering optimistic edits. */
    fun version(): Long = mutationVersion

    init {
        scope.launch {
            outboxDao.observeCount().collect { _pendingCount.value = it }
        }
    }

    fun observeGroup(groupId: Long): Flow<GroupSnapshotDto?> =
        combine(snapshots, _idMappings) { snapshots, mappings ->
            // Prefer the id the caller navigated with; fall back to the server id
            // only after the cache was re-keyed by migration/pull.
            snapshots[groupId]
                ?: mappings[mappingKey(EntityType.GROUP, groupId)]?.let(snapshots::get)
        }.distinctUntilChanged()

    /** Resolves a temp id to the server id once the create operation completed. */
    fun currentId(entityType: EntityType, id: Long): Long =
        if (id >= 0) id else _idMappings.value[mappingKey(entityType, id)] ?: id

    suspend fun load() {
        writeMutex.withLock {
            try {
                val cachedGroups = snapshotDao.getGroups()
                if (cachedGroups != null) {
                    _groups.value = decodeGroups(cachedGroups.json)
                }
                _snapshots.value = snapshotDao.all()
                    .mapNotNull { entity -> decodeSnapshot(entity) }
                    .associateBy { it.group.id }
                _idMappings.value = idMappingDao.all().associate { it.mappingKey to it.serverId }
            } finally {
                _isLoaded.value = true
            }
        }
    }

    suspend fun awaitLoaded() {
        isLoaded.first { it }
    }

    suspend fun replaceGroupsIfUnchanged(
        groups: List<GroupSummaryDto>,
        expectedVersion: Long,
    ): Boolean = writeMutex.withLock {
        if (mutationVersion != expectedVersion) return@withLock false
        applyGroups(groups)
        true
    }

    private suspend fun applyGroups(groups: List<GroupSummaryDto>) {
        val pendingGroupIds = outboxDao.all().map { it.groupId }.toSet()
        val serverIds = groups.map { it.id }.toSet()
        // Keep groups that still have queued work (including local-only groups
        // whose create has not reached the server yet) and drop everything else
        // the server no longer returns.
        val localOnly = _groups.value.filter {
            it.id !in serverIds && it.id in pendingGroupIds
        }
        val merged = (groups + localOnly).distinctBy { it.id }
        persistGroups(merged)
        _groups.value = merged

        val keep = (serverIds + localOnly.map { it.id }).toSet()
        _snapshots.value.keys
            .filter { it !in keep }
            .forEach { stale ->
                snapshotDao.delete(stale)
                _snapshots.update { it - stale }
            }
    }

    /**
     * Keeps cached data and unsynced operations tied to their owner. Signing in
     * as a different account wipes the cache; signing back in as the same
     * account preserves queued offline edits for retry.
     */
    suspend fun onUserChanged(userId: Long) {
        writeMutex.withLock {
            val previousOwner = metadataDao.get(OWNER_KEY)?.toLongOrNull()
            if (previousOwner != userId) {
                snapshotDao.deleteAll()
                snapshotDao.deleteGroups()
                outboxDao.deleteAll()
                idMappingDao.deleteAll()
                _snapshots.value = emptyMap()
                _idMappings.value = emptyMap()
                _groups.value = emptyList()
                mutationVersion++
            }
            metadataDao.put(LocalMetadataEntity(OWNER_KEY, userId.toString()))
        }
    }

    suspend fun groupSnapshot(groupId: Long): GroupSnapshotDto? {
        _snapshots.value[groupId]?.let { return it }
        loadSnapshot(groupId)?.let { loaded ->
            _snapshots.update { it + (groupId to loaded) }
            return loaded
        }
        val effective = currentId(EntityType.GROUP, groupId)
        _snapshots.value[effective]?.let { return it }
        return loadSnapshot(effective)?.also { loaded ->
            _snapshots.update { it + (effective to loaded) }
        }
    }

    /**
     * Pull result that is only applied when no local mutation happened while it
     * was in flight. Returns false when the caller should retry after pushing.
     */
    suspend fun replaceSnapshotIfUnchanged(
        snapshot: GroupSnapshotDto,
        expectedVersion: Long,
    ): Boolean = writeMutex.withLock {
        if (mutationVersion != expectedVersion) return@withLock false
        persistSnapshot(snapshot)
        _snapshots.update { it + (snapshot.group.id to snapshot) }
        true
    }

    suspend fun removeSnapshot(groupId: Long) {
        writeMutex.withLock {
            snapshotDao.delete(groupId)
            _snapshots.update { it - groupId }
        }
    }

    suspend fun createGroup(name: String, user: UserDto): GroupSummaryDto {
        val id = TempIds.next()
        val now = Instant.now().toString()
        val summary = GroupSummaryDto(
            id = id,
            name = name,
            createdBy = user.id,
            createdAt = now,
            updatedAt = now,
            memberCount = 1,
            isAdmin = true,
        )
        val snapshot = GroupSnapshotDto(
            group = GroupDto(id, name, user.id, now, now),
            members = listOf(GroupMemberDto(user.id, user.username, isCreator = true)),
            generatedAt = now,
        )
        val operation = PendingOperation(
            groupId = id,
            entityType = EntityType.GROUP,
            entityId = id,
            operation = OperationType.CREATE,
            payload = AppJson.encodeToString(
                CreateGroupRequest.serializer(),
                CreateGroupRequest(name),
            ),
        )

        writeMutex.withLock {
            val nextGroups = _groups.value + summary
            database.withTransaction {
                snapshotDao.put(
                    GroupSnapshotEntity(
                        groupId = id,
                        json = AppJson.encodeToString(GroupSnapshotDto.serializer(), snapshot),
                        syncedAt = System.currentTimeMillis(),
                    ),
                )
                persistGroups(nextGroups)
                outboxDao.insert(operation.toEntity())
            }
            mutationVersion++
            _groups.value = nextGroups
            _snapshots.update { it + (id to snapshot) }
        }
        return summary
    }

    /**
     * Applies [transform] to the cached snapshot and, in the same transaction,
     * inserts [operation] into the outbox.
     */
    suspend fun mutateGroup(
        groupId: Long,
        operation: PendingOperation,
        transform: (GroupSnapshotDto) -> GroupSnapshotDto,
    ) {
        writeMutex.withLock {
            val current = _snapshots.value[groupId]
                ?: loadSnapshot(groupId)
                ?: currentId(EntityType.GROUP, groupId).let { effective ->
                    _snapshots.value[effective] ?: loadSnapshot(effective)
                }
                ?: error("No cached snapshot for group $groupId")
            val snapshotGroupId = current.group.id
            val next = transform(current)
            check(next.group.id == snapshotGroupId) {
                "Snapshot transform changed the group id"
            }

            database.withTransaction {
                snapshotDao.put(
                    GroupSnapshotEntity(
                        groupId = snapshotGroupId,
                        json = AppJson.encodeToString(GroupSnapshotDto.serializer(), next),
                        syncedAt = System.currentTimeMillis(),
                    ),
                )
                outboxDao.insert(operation.copy(groupId = snapshotGroupId).toEntity())
            }
            mutationVersion++
            _snapshots.update { it + (snapshotGroupId to next) }
        }
    }

    suspend fun pendingOperations(): List<PendingOperation> =
        outboxDao.all().map(PendingOperation::fromEntity)

    suspend fun pendingCountForGroup(groupId: Long): Int = outboxDao.countForGroup(groupId)

    suspend fun completeOperation(sequence: Long) {
        outboxDao.delete(sequence)
    }

    suspend fun failOperation(sequence: Long, error: String) {
        outboxDao.recordFailure(sequence, error)
    }

    suspend fun dropOperationsForEntity(entityType: EntityType, entityId: Long) {
        outboxDao.deleteForEntity(entityType.name, entityId)
    }

    suspend fun dropOperationsForGroup(groupId: Long) {
        outboxDao.deleteForGroup(groupId)
    }

    suspend fun dropDependents(operation: PendingOperation) {
        when (operation.entityType) {
            EntityType.GROUP -> dropOperationsForGroup(operation.entityId)
            EntityType.SHOPPING_LIST ->
                pendingOperations()
                    .filter { it.parentId == operation.entityId }
                    .forEach { completeOperation(it.sequence) }
            else -> dropOperationsForEntity(operation.entityType, operation.entityId)
        }
    }

    suspend fun mapping(entityType: EntityType, localId: Long): Long? =
        idMappingDao.serverId(mappingKey(entityType, localId))

    suspend fun saveMapping(entityType: EntityType, localId: Long, serverId: Long) {
        val key = mappingKey(entityType, localId)
        idMappingDao.put(IdMappingEntity(key, serverId))
        _idMappings.update { it + (key to serverId) }
    }

    /**
     * Re-keys a locally created group after the server assigned its real id.
     * This keeps the open screen (and any later outbox entries) consistent
     * between the create response and the next snapshot pull.
     */
    suspend fun migrateGroup(localId: Long, serverId: Long) {
        if (localId == serverId) return
        writeMutex.withLock {
            val snapshot = _snapshots.value[localId] ?: loadSnapshot(localId)
            val migrated = snapshot?.copy(
                group = snapshot.group.copy(id = serverId),
                debts = snapshot.debts.map { it.withGroupId(serverId) },
                transactions = snapshot.transactions.map { it.copy(groupId = serverId) },
                shoppingLists = snapshot.shoppingLists.map { it.withGroupId(serverId) },
                recurringDebts = snapshot.recurringDebts.map { it.withGroupId(serverId) },
                markers = snapshot.markers.map { it.copy(groupId = serverId) },
            )
            val nextGroups = _groups.value.map { group ->
                if (group.id == localId) group.copy(id = serverId) else group
            }

            database.withTransaction {
                if (migrated != null) {
                    snapshotDao.delete(localId)
                    snapshotDao.put(
                        GroupSnapshotEntity(
                            groupId = serverId,
                            json = AppJson.encodeToString(
                                GroupSnapshotDto.serializer(),
                                migrated,
                            ),
                            syncedAt = System.currentTimeMillis(),
                        ),
                    )
                    persistGroups(nextGroups)
                }
                // Dependents created before the group was pushed must follow the
                // new id even if the local snapshot was already gone.
                outboxDao.reassignGroup(localId, serverId)
                idMappingDao.put(
                    IdMappingEntity(mappingKey(EntityType.GROUP, localId), serverId),
                )
            }
            _idMappings.update {
                it + (mappingKey(EntityType.GROUP, localId) to serverId)
            }
            mutationVersion++
            if (migrated != null) {
                _snapshots.update { snapshots ->
                    snapshots - localId + (serverId to migrated)
                }
                _groups.value = nextGroups
            }
        }
    }

    private suspend fun persistSnapshot(snapshot: GroupSnapshotDto) {
        snapshotDao.put(
            GroupSnapshotEntity(
                groupId = snapshot.group.id,
                json = AppJson.encodeToString(GroupSnapshotDto.serializer(), snapshot),
                syncedAt = System.currentTimeMillis(),
            ),
        )
    }

    private suspend fun loadSnapshot(groupId: Long): GroupSnapshotDto? =
        snapshotDao.get(groupId)?.let { decodeSnapshot(it) }

    private suspend fun persistGroups(groups: List<GroupSummaryDto>) {
        snapshotDao.putGroups(
            GroupsCacheEntity(
                json = AppJson.encodeToString(
                    kotlinx.serialization.builtins.ListSerializer(GroupSummaryDto.serializer()),
                    groups,
                ),
                syncedAt = System.currentTimeMillis(),
            ),
        )
    }

    private fun decodeSnapshot(entity: GroupSnapshotEntity): GroupSnapshotDto? = runCatching {
        AppJson.decodeFromString(GroupSnapshotDto.serializer(), entity.json)
    }.getOrNull()

    private fun decodeGroups(json: String): List<GroupSummaryDto> = runCatching {
        AppJson.decodeFromString(
            kotlinx.serialization.builtins.ListSerializer(GroupSummaryDto.serializer()),
            json,
        )
    }.getOrDefault(emptyList())

    private fun mappingKey(entityType: EntityType, localId: Long): String =
        "${entityType.name}:$localId"

    private companion object {
        const val OWNER_KEY = "owner_user_id"
    }

    private fun DebtSnapshotDto.withGroupId(groupId: Long): DebtSnapshotDto =
        copy(debt = debt.copy(groupId = groupId))

    private fun ShoppingListSnapshotDto.withGroupId(groupId: Long): ShoppingListSnapshotDto =
        copy(list = list.copy(groupId = groupId))

    private fun RecurringDebtSnapshotDto.withGroupId(groupId: Long): RecurringDebtSnapshotDto =
        copy(debt = debt.copy(groupId = groupId))
}
