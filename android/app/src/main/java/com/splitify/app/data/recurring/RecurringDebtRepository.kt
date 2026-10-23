package com.splitify.app.data.recurring

import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.cache.EntityType
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.cache.OperationType
import com.splitify.app.data.cache.PendingOperation
import com.splitify.app.data.cache.TempIds
import com.splitify.app.data.dto.CreateRecurringDebtRequest
import com.splitify.app.data.dto.GeneratedInstanceDto
import com.splitify.app.data.dto.GroupSnapshotDto
import com.splitify.app.data.dto.RecurringDebtDto
import com.splitify.app.data.dto.RecurringDebtMemberDto
import com.splitify.app.data.dto.RecurringDebtSnapshotDto
import com.splitify.app.data.dto.ToggleRecurringRequest
import com.splitify.app.data.dto.UpdateRecurringDebtRequest
import com.splitify.app.data.network.AppJson
import com.splitify.app.data.sync.SyncManager
import java.math.BigDecimal
import java.math.RoundingMode
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class RecurringDebtRepository(
    private val localStore: LocalStore,
    private val syncManager: SyncManager,
    private val session: SessionManager,
) {

    fun observeDebts(groupId: Long): Flow<List<RecurringDebtDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.recurringDebts.orEmpty()
                .map { it.debt }
                .sortedWith(
                    compareByDescending<RecurringDebtDto> { it.createdAt }.thenByDescending { it.id },
                )
        }

    fun observeShares(groupId: Long): Flow<Map<Long, List<RecurringDebtMemberDto>>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.recurringDebts.orEmpty().associate { it.debt.id to it.shares }
        }

    fun observeMemberIds(groupId: Long): Flow<Map<Long, List<Long>>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.recurringDebts.orEmpty().associate { it.debt.id to it.memberIds }
        }

    fun observeInstances(groupId: Long): Flow<Map<Long, List<GeneratedInstanceDto>>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.recurringDebts.orEmpty().associate { it.debt.id to it.instances }
        }

    suspend fun createRecurringDebt(
        groupId: Long,
        request: CreateRecurringDebtRequest,
    ): Result<Long> = runCatching {
        val user = session.requireUser()
        val snapshot = requireSnapshot(groupId)
        val id = TempIds.next()
        val now = Instant.now().toString()
        val debt = RecurringDebtDto(
            id = id,
            groupId = groupId,
            createdBy = user.id,
            creatorUsername = user.username,
            name = request.name,
            amount = request.amount,
            frequency = request.frequency,
            startDate = request.startDate,
            endDate = request.endDate,
            nextGenerationDate = request.startDate,
            isActive = true,
            createdAt = now,
            updatedAt = now,
            isCreator = true,
            status = statusFor(true, request.endDate),
        )
        val entry = RecurringDebtSnapshotDto(
            debt = debt,
            memberIds = request.memberIds,
            shares = sharesFor(snapshot, request.memberIds, request.amount),
        )
        enqueue(
            groupId = groupId,
            entityId = id,
            operation = OperationType.CREATE,
            payload = AppJson.encodeToString(CreateRecurringDebtRequest.serializer(), request),
            transform = { current ->
                current.copy(recurringDebts = listOf(entry) + current.recurringDebts)
            },
        )
        id
    }

    suspend fun updateRecurringDebt(
        groupId: Long,
        recurringDebtId: Long,
        request: UpdateRecurringDebtRequest,
    ): Result<Unit> = runCatching {
        val snapshot = requireSnapshot(groupId)
        val effectiveId = snapshot.recurringDebts
            .firstOrNull { it.debt.id == recurringDebtId }
            ?.debt
            ?.id
            ?: localStore.currentId(EntityType.RECURRING_DEBT, recurringDebtId)
        enqueue(
            groupId = groupId,
            entityId = effectiveId,
            operation = OperationType.UPDATE,
            payload = AppJson.encodeToString(UpdateRecurringDebtRequest.serializer(), request),
            transform = { current ->
                current.copy(
                    recurringDebts = current.recurringDebts.map { entry ->
                        if (entry.debt.id != effectiveId) {
                            entry
                        } else {
                            entry.copy(
                                debt = entry.debt.copy(
                                    name = request.name,
                                    amount = request.amount,
                                    frequency = request.frequency,
                                    endDate = request.endDate,
                                    isActive = request.isActive,
                                    status = statusFor(request.isActive, request.endDate),
                                    updatedAt = Instant.now().toString(),
                                ),
                                memberIds = request.memberIds,
                                shares = sharesFor(snapshot, request.memberIds, request.amount),
                            )
                        }
                    },
                )
            },
        )
    }

    suspend fun deleteRecurringDebt(groupId: Long, recurringDebtId: Long): Result<Unit> =
        runCatching {
            val snapshot = requireSnapshot(groupId)
            val effectiveId = snapshot.recurringDebts
                .firstOrNull { it.debt.id == recurringDebtId }
                ?.debt
                ?.id
                ?: localStore.currentId(EntityType.RECURRING_DEBT, recurringDebtId)
            enqueue(
                groupId = groupId,
                entityId = effectiveId,
                operation = OperationType.DELETE,
                transform = { current ->
                    current.copy(
                        recurringDebts = current.recurringDebts.filterNot {
                            it.debt.id == effectiveId
                        },
                    )
                },
            )
        }

    suspend fun toggleRecurringDebt(groupId: Long, recurringDebtId: Long): Result<Unit> =
        runCatching {
            val entry = requireSnapshot(groupId).recurringDebts
                .firstOrNull { it.debt.id == recurringDebtId }
                ?: requireSnapshot(groupId).recurringDebts.firstOrNull {
                    it.debt.id == localStore.currentId(EntityType.RECURRING_DEBT, recurringDebtId)
                }
                ?: error("Recurring debt $recurringDebtId is not cached")
            val effectiveId = entry.debt.id
            val desired = !entry.debt.isActive
            enqueue(
                groupId = groupId,
                entityId = effectiveId,
                operation = OperationType.TOGGLE_RECURRING,
                payload = AppJson.encodeToString(
                    ToggleRecurringRequest.serializer(),
                    ToggleRecurringRequest(desired),
                ),
                transform = { current ->
                    current.copy(
                        recurringDebts = current.recurringDebts.map { existing ->
                            if (existing.debt.id != effectiveId) {
                                existing
                            } else {
                                existing.copy(
                                    debt = existing.debt.copy(
                                        isActive = desired,
                                        status = statusFor(desired, existing.debt.endDate),
                                        updatedAt = Instant.now().toString(),
                                    ),
                                )
                            }
                        },
                    )
                },
            )
        }

    suspend fun generateRecurringDebt(groupId: Long, recurringDebtId: Long): Result<Unit> =
        runCatching {
            val snapshot = requireSnapshot(groupId)
            val effectiveId = snapshot.recurringDebts
                .firstOrNull { it.debt.id == recurringDebtId }
                ?.debt
                ?.id
                ?: localStore.currentId(EntityType.RECURRING_DEBT, recurringDebtId)
            localStore.mutateGroup(
                groupId = groupId,
                operation = PendingOperation(
                    groupId = groupId,
                    entityType = EntityType.RECURRING_DEBT,
                    entityId = effectiveId,
                    operation = OperationType.GENERATE_RECURRING,
                ),
                transform = { it },
            )
            syncManager.requestSync()
        }

    fun refresh() {
        syncManager.requestSync()
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
                entityType = EntityType.RECURRING_DEBT,
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
    ): List<RecurringDebtMemberDto> {
        if (memberIds.isEmpty()) return emptyList()
        val share = runCatching {
            BigDecimal(amount)
                .divide(BigDecimal(memberIds.size), 2, RoundingMode.HALF_UP)
                .toPlainString()
        }.getOrDefault(amount)
        val usernames = snapshot.members.associate { it.id to it.username }
        return memberIds.map { userId ->
            RecurringDebtMemberDto(
                userId = userId,
                username = usernames[userId] ?: "",
                shareAmount = share,
            )
        }
    }

    private fun statusFor(isActive: Boolean, endDate: String?): String {
        if (!isActive) return "Paused"
        val parsedEnd = endDate?.let { runCatching { LocalDate.parse(it) }.getOrNull() }
        return if (parsedEnd != null && LocalDate.now(ZoneOffset.UTC).isAfter(parsedEnd)) {
            "Expired"
        } else {
            "Active"
        }
    }

    private suspend fun requireSnapshot(groupId: Long): GroupSnapshotDto =
        localStore.groupSnapshot(groupId)
            ?: error("Group $groupId is not cached yet")
}
