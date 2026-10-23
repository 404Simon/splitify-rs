package com.splitify.app.data.debts

import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.cache.EntityType
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.cache.OperationType
import com.splitify.app.data.cache.PendingOperation
import com.splitify.app.data.cache.TempIds
import com.splitify.app.data.dto.CreateDebtRequest
import com.splitify.app.data.dto.DebtSnapshotDto
import com.splitify.app.data.dto.GroupSnapshotDto
import com.splitify.app.data.dto.SharedDebtDto
import com.splitify.app.data.dto.UserShareDto
import com.splitify.app.data.network.AppJson
import com.splitify.app.data.sync.SyncManager
import java.math.BigDecimal
import java.math.RoundingMode
import java.time.Instant
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class DebtRepository(
    private val localStore: LocalStore,
    private val syncManager: SyncManager,
    private val session: SessionManager,
) {

    fun observeDebts(groupId: Long): Flow<List<SharedDebtDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.debts.orEmpty()
                .map { it.debt }
                .sortedWith(compareByDescending<SharedDebtDto> { it.createdAt }.thenByDescending { it.id })
        }

    fun observeShares(groupId: Long): Flow<Map<Long, List<UserShareDto>>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.debts.orEmpty().associate { it.debt.id to it.shares }
        }

    suspend fun createDebt(
        groupId: Long,
        name: String,
        amount: String,
        memberIds: List<Long>,
    ): Result<Long> = runCatching {
        val user = session.requireUser()
        val snapshot = requireSnapshot(groupId)
        val id = TempIds.next()
        val now = Instant.now().toString()
        val debt = SharedDebtDto(
            id = id,
            groupId = groupId,
            createdBy = user.id,
            creatorUsername = user.username,
            name = name,
            amount = amount,
            createdAt = now,
            updatedAt = now,
            isCreator = true,
        )
        val entry = DebtSnapshotDto(
            debt = debt,
            participantIds = memberIds,
            shares = sharesFor(snapshot, memberIds, amount),
        )
        val request = CreateDebtRequest(name = name, amount = amount, memberIds = memberIds)
        enqueue(
            groupId = groupId,
            entityId = id,
            operation = OperationType.CREATE,
            payload = AppJson.encodeToString(CreateDebtRequest.serializer(), request),
            transform = { current -> current.copy(debts = listOf(entry) + current.debts) },
        )
        id
    }

    suspend fun updateDebt(
        groupId: Long,
        debtId: Long,
        name: String,
        amount: String,
        memberIds: List<Long>,
    ): Result<Unit> = runCatching {
        val snapshot = requireSnapshot(groupId)
        val effectiveDebtId = snapshot.debts.firstOrNull { it.debt.id == debtId }?.debt?.id
            ?: localStore.currentId(EntityType.DEBT, debtId)
        val request = CreateDebtRequest(name = name, amount = amount, memberIds = memberIds)
        enqueue(
            groupId = groupId,
            entityId = effectiveDebtId,
            operation = OperationType.UPDATE,
            payload = AppJson.encodeToString(CreateDebtRequest.serializer(), request),
            transform = { current ->
                current.copy(
                    debts = current.debts.map { entry ->
                        if (entry.debt.id != effectiveDebtId) {
                            entry
                        } else {
                            DebtSnapshotDto(
                                debt = entry.debt.copy(
                                    name = name,
                                    amount = amount,
                                    updatedAt = Instant.now().toString(),
                                ),
                                participantIds = memberIds,
                                shares = sharesFor(snapshot, memberIds, amount),
                            )
                        }
                    },
                )
            },
        )
    }

    suspend fun deleteDebt(groupId: Long, debtId: Long): Result<Unit> = runCatching {
        val snapshot = requireSnapshot(groupId)
        val effectiveDebtId = snapshot.debts.firstOrNull { it.debt.id == debtId }?.debt?.id
            ?: localStore.currentId(EntityType.DEBT, debtId)
        enqueue(
            groupId = groupId,
            entityId = effectiveDebtId,
            operation = OperationType.DELETE,
            transform = { current ->
                current.copy(
                    debts = current.debts.filterNot { it.debt.id == effectiveDebtId },
                )
            },
        )
    }

    private suspend fun enqueue(
        groupId: Long,
        entityId: Long,
        operation: OperationType,
        payload: String = "{}",
        transform: (GroupSnapshotDto) -> GroupSnapshotDto,
    ) {
        localStore.mutateGroup(
            groupId = groupId,
            operation = PendingOperation(
                groupId = groupId,
                entityType = EntityType.DEBT,
                entityId = entityId,
                operation = operation,
                payload = payload,
            ),
            transform = transform,
        )
        syncManager.requestSync()
    }

    private fun sharesFor(
        snapshot: GroupSnapshotDto,
        memberIds: List<Long>,
        amount: String,
    ): List<UserShareDto> {
        if (memberIds.isEmpty()) return emptyList()
        val share = runCatching {
            BigDecimal(amount)
                .divide(BigDecimal(memberIds.size), 2, RoundingMode.HALF_UP)
                .toPlainString()
        }.getOrDefault(amount)
        val usernames = snapshot.members.associate { it.id to it.username }
        return memberIds.map { userId ->
            UserShareDto(
                userId = userId,
                username = usernames[userId] ?: "",
                shareAmount = share,
            )
        }
    }

    private suspend fun requireSnapshot(groupId: Long): GroupSnapshotDto =
        localStore.groupSnapshot(groupId)
            ?: error("Group $groupId is not cached yet")
}
