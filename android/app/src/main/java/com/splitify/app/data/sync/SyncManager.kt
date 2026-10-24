package com.splitify.app.data.sync

import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.cache.EntityType
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.cache.OperationType
import com.splitify.app.data.cache.PendingOperation
import com.splitify.app.data.config.ServerConfig
import com.splitify.app.data.dto.CreateDebtRequest
import com.splitify.app.data.dto.CreateGroupRequest
import com.splitify.app.data.dto.CreateMarkerRequest
import com.splitify.app.data.dto.CreateRecurringDebtRequest
import com.splitify.app.data.dto.CreateShoppingItemRequest
import com.splitify.app.data.dto.CreateShoppingListRequest
import com.splitify.app.data.dto.CreateTransactionRequest
import com.splitify.app.data.dto.ToggleItemRequest
import com.splitify.app.data.dto.ToggleRecurringRequest
import com.splitify.app.data.dto.UpdateGroupRequest
import com.splitify.app.data.dto.UpdateRecurringDebtRequest
import com.splitify.app.data.network.ApiException
import com.splitify.app.data.network.AppJson
import com.splitify.app.data.network.SyncApi
import com.splitify.app.data.network.userMessage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Pushes the local outbox, then pulls a fresh snapshot for every group that has
 * no pending operations.
 *
 * The order matters: pushing first means a group with local changes is never
 * overwritten by stale server data, and pulling afterwards replaces temp ids
 * with the ids assigned by the server.
 */
class SyncManager(
    private val api: SyncApi,
    private val store: LocalStore,
    private val session: SessionManager,
    private val connectivity: ConnectivityObserver,
    private val serverConfig: ServerConfig,
    private val scope: CoroutineScope,
    private val scheduleRetry: () -> Unit = {},
) {

    private val mutex = Mutex()

    /**
     * Conflated request channel: bursts of requests (tab switches, several local
     * mutations) collapse into one run, and a request that arrives mid-flight is
     * guaranteed a follow-up pass.
     */
    private val requests = MutableSharedFlow<Unit>(
        extraBufferCapacity = 1,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    private val _status = MutableStateFlow<SyncStatus>(SyncStatus.Idle)
    val status: StateFlow<SyncStatus> = _status.asStateFlow()

    private var started = false

    init {
        scope.launch {
            requests.collect { sync() }
        }
    }

    fun start() {
        if (started) return
        started = true
        scope.launch {
            connectivity.isOnline.collect { online ->
                when {
                    !online -> _status.value = SyncStatus.Offline
                    session.token.value != null -> requestSync()
                    else -> _status.value = SyncStatus.Idle
                }
            }
        }
    }

    /** Fire-and-forget sync used after mutations and pulls. */
    fun requestSync() {
        requests.tryEmit(Unit)
    }

    /** Blocking sync used by WorkManager. Returns false when a retry is needed. */
    suspend fun syncNow(): Boolean = sync()

    /**
     * Fetches one group immediately, e.g. right after accepting an invite so the
     * detail screen has data before navigation completes.
     */
    suspend fun syncGroup(groupId: Long) {
        session.awaitReady()
        store.awaitLoaded()
        mutex.withLock {
            if (serverConfig.current() == null) return@withLock
            if (session.token.value == null || !connectivity.isOnline.value) return@withLock
            try {
                val version = store.version()
                store.replaceSnapshotIfUnchanged(api.groupSnapshot(groupId), version)
            } catch (error: CancellationException) {
                throw error
            } catch (error: ApiException.Http) {
                if (error.isUnauthorized) session.clear()
            } catch (_: Exception) {
                // The regular sync/WorkManager retry will pick this up.
            }
        }
    }

    private suspend fun sync(): Boolean {
        // Background workers can start before Application finished hydrating the
        // session/cache; wait instead of silently skipping the run.
        session.awaitReady()
        store.awaitLoaded()
        return syncLocked()
    }

    private suspend fun syncLocked(): Boolean = mutex.withLock {
        // Until the user has entered a server address there is nowhere to sync to.
        if (serverConfig.current() == null) {
            _status.value = SyncStatus.Idle
            return@withLock true
        }
        if (session.token.value == null) {
            _status.value = SyncStatus.Idle
            return@withLock true
        }
        if (!connectivity.isOnline.value) {
            _status.value = SyncStatus.Offline
            return@withLock false
        }

        _status.value = SyncStatus.Syncing
        var transientFailure: String? = null
        var permanentFailure: String? = null

        try {
            for (operation in store.pendingOperations()) {
                when (val result = pushOperation(operation)) {
                    PushResult.Success -> store.completeOperation(operation.sequence)

                    is PushResult.Transient -> {
                        store.failOperation(operation.sequence, result.message)
                        transientFailure = result.message
                        break
                    }

                    is PushResult.Permanent -> {
                        store.failOperation(operation.sequence, result.message)
                        store.completeOperation(operation.sequence)
                        if (operation.operation == OperationType.CREATE) {
                            store.dropDependents(operation)
                        }
                        permanentFailure = result.message
                    }
                }
            }

            if (transientFailure != null) {
                _status.value = if (connectivity.isOnline.value) {
                    SyncStatus.Error(transientFailure)
                } else {
                    SyncStatus.Offline
                }
                scheduleRetry()
                return@withLock false
            }

            if (!pullGroups()) {
                _status.value = SyncStatus.Idle
                return@withLock false
            }

            _status.value = permanentFailure?.let(SyncStatus::Error) ?: SyncStatus.Idle
            true
        } catch (error: ApiException.Unauthorized) {
            session.clear()
            _status.value = SyncStatus.Idle
            false
        } catch (error: ApiException.Http) {
            if (error.isUnauthorized) {
                session.clear()
                _status.value = SyncStatus.Idle
            } else {
                _status.value = SyncStatus.Error(error.userMessage())
                scheduleRetry()
            }
            false
        } catch (error: ApiException.Network) {
            _status.value = if (connectivity.isOnline.value) {
                SyncStatus.Error(error.userMessage())
            } else {
                SyncStatus.Offline
            }
            scheduleRetry()
            false
        } catch (error: Exception) {
            _status.value = SyncStatus.Error(error.userMessage())
            false
        }
    }

    private suspend fun pullGroups(): Boolean {
        val groupsVersion = store.version()
        val groups = api.listGroups()
        if (!store.replaceGroupsIfUnchanged(groups, groupsVersion)) return false

        for (group in groups) {
            // Capture the version first: a mutation that happened just before
            // this point is then visible to the pending check below, and one
            // that happens later invalidates the version check after the fetch.
            val version = store.version()
            if (store.pendingCountForGroup(group.id) > 0) continue
            try {
                val snapshot = api.groupSnapshot(group.id)
                store.replaceSnapshotIfUnchanged(snapshot, version)
            } catch (error: ApiException.Http) {
                if (error.statusCode == 404 || error.statusCode == 403) {
                    store.removeSnapshot(group.id)
                } else {
                    throw error
                }
            }
        }
        return true
    }

    private suspend fun pushOperation(operation: PendingOperation): PushResult = try {
        when (operation.operation) {
            OperationType.CREATE -> pushCreate(operation)
            OperationType.UPDATE -> pushUpdate(operation)
            OperationType.DELETE -> pushDelete(operation)
            OperationType.TOGGLE_ITEM -> pushToggleItem(operation)
            OperationType.TOGGLE_RECURRING -> pushToggleRecurring(operation)
            OperationType.GENERATE_RECURRING -> api.generateRecurringDebt(
                id = resolve(operation.entityType, operation.entityId) ?: return PushResult.Success,
                key = operation.idempotencyKey,
            )
        }
        PushResult.Success
    } catch (error: ApiException.Http) {
        if (error.isUnauthorized) throw error
        if (error.statusCode in TRANSIENT_STATUS_CODES) {
            PushResult.Transient(error.userMessage())
        } else {
            PushResult.Permanent(error.userMessage())
        }
    } catch (error: ApiException.Unauthorized) {
        throw error
    } catch (error: ApiException.Network) {
        PushResult.Transient(error.userMessage())
    } catch (error: ApiException.Parse) {
        PushResult.Permanent(error.userMessage())
    } catch (error: Exception) {
        PushResult.Permanent(error.userMessage())
    }

    private suspend fun pushCreate(operation: PendingOperation) {
        val serverId = when (operation.entityType) {
            EntityType.GROUP -> {
                val body = AppJson.decodeFromString(
                    CreateGroupRequest.serializer(),
                    operation.payload,
                )
                api.createGroup(body.name, operation.idempotencyKey)
            }

            EntityType.DEBT -> {
                val groupId = resolve(EntityType.GROUP, operation.groupId) ?: return
                api.createDebt(
                    groupId,
                    AppJson.decodeFromString(CreateDebtRequest.serializer(), operation.payload),
                    operation.idempotencyKey,
                )
            }

            EntityType.TRANSACTION -> {
                val groupId = resolve(EntityType.GROUP, operation.groupId) ?: return
                api.createTransaction(
                    groupId,
                    AppJson.decodeFromString(
                        CreateTransactionRequest.serializer(),
                        operation.payload,
                    ),
                    operation.idempotencyKey,
                )
            }

            EntityType.SHOPPING_LIST -> {
                val groupId = resolve(EntityType.GROUP, operation.groupId) ?: return
                api.createShoppingList(
                    groupId,
                    AppJson.decodeFromString(
                        CreateShoppingListRequest.serializer(),
                        operation.payload,
                    ),
                    operation.idempotencyKey,
                )
            }

            EntityType.SHOPPING_ITEM -> {
                val listId = operation.parentId?.let { resolve(EntityType.SHOPPING_LIST, it) }
                    ?: return
                api.createShoppingItem(
                    listId,
                    AppJson.decodeFromString(
                        CreateShoppingItemRequest.serializer(),
                        operation.payload,
                    ),
                    operation.idempotencyKey,
                )
            }

            EntityType.RECURRING_DEBT -> {
                val groupId = resolve(EntityType.GROUP, operation.groupId) ?: return
                api.createRecurringDebt(
                    groupId,
                    AppJson.decodeFromString(
                        CreateRecurringDebtRequest.serializer(),
                        operation.payload,
                    ),
                    operation.idempotencyKey,
                )
            }

            EntityType.MARKER -> {
                val groupId = resolve(EntityType.GROUP, operation.groupId) ?: return
                api.createMarker(
                    groupId,
                    AppJson.decodeFromString(CreateMarkerRequest.serializer(), operation.payload),
                    operation.idempotencyKey,
                )
            }
        }

        store.saveMapping(operation.entityType, operation.entityId, serverId)
        if (operation.entityType == EntityType.GROUP) {
            store.migrateGroup(operation.entityId, serverId)
        }
    }

    private suspend fun pushUpdate(operation: PendingOperation) {
        val id = resolve(operation.entityType, operation.entityId) ?: return
        when (operation.entityType) {
            EntityType.GROUP -> api.updateGroup(
                id,
                AppJson.decodeFromString(UpdateGroupRequest.serializer(), operation.payload),
                operation.idempotencyKey,
            )

            EntityType.DEBT -> api.updateDebt(
                id,
                AppJson.decodeFromString(CreateDebtRequest.serializer(), operation.payload),
                operation.idempotencyKey,
            )

            EntityType.TRANSACTION -> api.updateTransaction(
                id,
                AppJson.decodeFromString(
                    CreateTransactionRequest.serializer(),
                    operation.payload,
                ),
                operation.idempotencyKey,
            )

            EntityType.SHOPPING_LIST -> api.updateShoppingList(
                id,
                AppJson.decodeFromString(
                    CreateShoppingListRequest.serializer(),
                    operation.payload,
                ),
                operation.idempotencyKey,
            )

            EntityType.SHOPPING_ITEM -> {
                val listId = operation.parentId?.let { resolve(EntityType.SHOPPING_LIST, it) }
                    ?: return
                api.updateShoppingItem(
                    listId,
                    id,
                    AppJson.decodeFromString(
                        CreateShoppingItemRequest.serializer(),
                        operation.payload,
                    ),
                    operation.idempotencyKey,
                )
            }

            EntityType.RECURRING_DEBT -> api.updateRecurringDebt(
                id,
                AppJson.decodeFromString(
                    UpdateRecurringDebtRequest.serializer(),
                    operation.payload,
                ),
                operation.idempotencyKey,
            )

            EntityType.MARKER -> api.updateMarker(
                id,
                AppJson.decodeFromString(CreateMarkerRequest.serializer(), operation.payload),
                operation.idempotencyKey,
            )
        }
    }

    private suspend fun pushDelete(operation: PendingOperation) {
        val id = resolve(operation.entityType, operation.entityId) ?: return
        when (operation.entityType) {
            EntityType.GROUP -> api.deleteGroup(id, operation.idempotencyKey)
            EntityType.DEBT -> api.deleteDebt(id, operation.idempotencyKey)
            EntityType.TRANSACTION -> api.deleteTransaction(id, operation.idempotencyKey)
            EntityType.SHOPPING_LIST -> api.deleteShoppingList(id, operation.idempotencyKey)
            EntityType.SHOPPING_ITEM -> {
                val listId = operation.parentId?.let { resolve(EntityType.SHOPPING_LIST, it) }
                    ?: return
                api.deleteShoppingItem(listId, id, operation.idempotencyKey)
            }

            EntityType.RECURRING_DEBT -> api.deleteRecurringDebt(id, operation.idempotencyKey)
            EntityType.MARKER -> api.deleteMarker(id, operation.idempotencyKey)
        }
    }

    private suspend fun pushToggleItem(operation: PendingOperation) {
        val itemId = resolve(EntityType.SHOPPING_ITEM, operation.entityId) ?: return
        val listId = operation.parentId?.let { resolve(EntityType.SHOPPING_LIST, it) } ?: return
        val body = AppJson.decodeFromString(ToggleItemRequest.serializer(), operation.payload)
        api.toggleShoppingItem(listId, itemId, body.isCompleted, operation.idempotencyKey)
    }

    private suspend fun pushToggleRecurring(operation: PendingOperation) {
        val id = resolve(EntityType.RECURRING_DEBT, operation.entityId) ?: return
        val body = AppJson.decodeFromString(
            ToggleRecurringRequest.serializer(),
            operation.payload,
        )
        api.toggleRecurringDebt(id, body.isActive, operation.idempotencyKey)
    }

    /**
     * Resolves a temp id through the mapping table. A missing mapping for a
     * negative id means the parent create failed permanently, so the dependent
     * operation is skipped (treated as success).
     */
    private suspend fun resolve(entityType: EntityType, id: Long): Long? =
        if (id >= 0) id else store.mapping(entityType, id)

    private sealed interface PushResult {
        data object Success : PushResult
        data class Transient(val message: String) : PushResult
        data class Permanent(val message: String) : PushResult
    }

    private companion object {
        val TRANSIENT_STATUS_CODES = setOf(408, 425, 429)
    }
}
