package com.splitify.app.data.transactions

import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.cache.EntityType
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.cache.OperationType
import com.splitify.app.data.cache.PendingOperation
import com.splitify.app.data.cache.TempIds
import com.splitify.app.data.domain.BalancesCalculator
import com.splitify.app.data.dto.CreateTransactionRequest
import com.splitify.app.data.dto.GroupSnapshotDto
import com.splitify.app.data.dto.TransactionDto
import com.splitify.app.data.dto.UserBalanceDto
import com.splitify.app.data.network.AppJson
import com.splitify.app.data.sync.SyncManager
import java.time.Instant
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class TransactionRepository(
    private val localStore: LocalStore,
    private val syncManager: SyncManager,
    private val session: SessionManager,
) {

    fun observeTransactions(groupId: Long): Flow<List<TransactionDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.transactions.orEmpty()
                .sortedWith(
                    compareByDescending<TransactionDto> { it.createdAt }
                        .thenByDescending { it.id },
                )
        }

    fun observeBalances(groupId: Long): Flow<List<UserBalanceDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            if (snapshot == null) {
                emptyList()
            } else {
                BalancesCalculator.calculate(
                    members = snapshot.members,
                    debts = snapshot.debts,
                    transactions = snapshot.transactions,
                )
            }
        }

    suspend fun createTransaction(
        groupId: Long,
        recipientId: Long,
        amount: String,
        description: String?,
    ): Result<Long> = runCatching {
        val user = session.requireUser()
        val snapshot = requireSnapshot(groupId)
        val id = TempIds.next()
        val now = Instant.now().toString()
        val transaction = TransactionDto(
            id = id,
            groupId = groupId,
            payerId = user.id,
            payerUsername = user.username,
            recipientId = recipientId,
            recipientUsername = snapshot.members.firstOrNull { it.id == recipientId }?.username.orEmpty(),
            amount = amount,
            description = description,
            createdAt = now,
            updatedAt = now,
        )
        val request = CreateTransactionRequest(
            recipientId = recipientId,
            amount = amount,
            description = description,
        )
        enqueue(
            groupId = groupId,
            entityId = id,
            operation = OperationType.CREATE,
            payload = AppJson.encodeToString(CreateTransactionRequest.serializer(), request),
            transform = { current ->
                current.copy(transactions = listOf(transaction) + current.transactions)
            },
        )
        id
    }

    suspend fun updateTransaction(
        groupId: Long,
        transactionId: Long,
        recipientId: Long,
        amount: String,
        description: String?,
    ): Result<Unit> = runCatching {
        val snapshot = requireSnapshot(groupId)
        val effectiveTransactionId = snapshot.transactions
            .firstOrNull { it.id == transactionId }
            ?.id
            ?: localStore.currentId(EntityType.TRANSACTION, transactionId)
        val recipientUsername =
            snapshot.members.firstOrNull { it.id == recipientId }?.username.orEmpty()
        val request = CreateTransactionRequest(
            recipientId = recipientId,
            amount = amount,
            description = description,
        )
        enqueue(
            groupId = groupId,
            entityId = effectiveTransactionId,
            operation = OperationType.UPDATE,
            payload = AppJson.encodeToString(CreateTransactionRequest.serializer(), request),
            transform = { current ->
                current.copy(
                    transactions = current.transactions.map { transaction ->
                        if (transaction.id != effectiveTransactionId) {
                            transaction
                        } else {
                            transaction.copy(
                                recipientId = recipientId,
                                recipientUsername = recipientUsername,
                                amount = amount,
                                description = description,
                                updatedAt = Instant.now().toString(),
                            )
                        }
                    },
                )
            },
        )
    }

    suspend fun deleteTransaction(groupId: Long, transactionId: Long): Result<Unit> = runCatching {
        val snapshot = requireSnapshot(groupId)
        val effectiveTransactionId = snapshot.transactions
            .firstOrNull { it.id == transactionId }
            ?.id
            ?: localStore.currentId(EntityType.TRANSACTION, transactionId)
        enqueue(
            groupId = groupId,
            entityId = effectiveTransactionId,
            operation = OperationType.DELETE,
            transform = { current ->
                current.copy(
                    transactions = current.transactions.filterNot {
                        it.id == effectiveTransactionId
                    },
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
                entityType = EntityType.TRANSACTION,
                entityId = entityId,
                operation = operation,
                payload = payload,
            ),
            transform = transform,
        )
        syncManager.requestSync()
    }

    private suspend fun requireSnapshot(groupId: Long): GroupSnapshotDto =
        localStore.groupSnapshot(groupId)
            ?: error("Group $groupId is not cached yet")
}
