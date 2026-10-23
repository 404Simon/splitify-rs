package com.splitify.app.data.cache

import com.splitify.app.data.local.entity.OutboxOperationEntity
import java.util.UUID
import java.util.concurrent.atomic.AtomicLong

/** Entity kinds that can be queued for sync. */
enum class EntityType {
    GROUP,
    DEBT,
    TRANSACTION,
    SHOPPING_LIST,
    SHOPPING_ITEM,
    RECURRING_DEBT,
    MARKER,
}

/** Mutations understood by [com.splitify.app.data.sync.SyncManager]. */
enum class OperationType {
    CREATE,
    UPDATE,
    DELETE,
    TOGGLE_ITEM,
    TOGGLE_RECURRING,
    GENERATE_RECURRING,
}

/**
 * A queued mutation. [entityId] and [parentId] may be negative temp ids; the
 * sync engine resolves them through the id-mapping table before sending.
 */
data class PendingOperation(
    val sequence: Long = 0,
    val idempotencyKey: String = UUID.randomUUID().toString(),
    val groupId: Long,
    val entityType: EntityType,
    val entityId: Long,
    val parentId: Long? = null,
    val operation: OperationType,
    val payload: String = "{}",
    val createdAt: Long = System.currentTimeMillis(),
    val attempts: Int = 0,
    val lastError: String? = null,
) {
    fun toEntity(): OutboxOperationEntity = OutboxOperationEntity(
        sequence = sequence,
        idempotencyKey = idempotencyKey,
        groupId = groupId,
        entityType = entityType.name,
        entityId = entityId,
        parentId = parentId,
        operation = operation.name,
        payload = payload,
        createdAt = createdAt,
        attempts = attempts,
        lastError = lastError,
    )

    companion object {
        fun fromEntity(entity: OutboxOperationEntity): PendingOperation = PendingOperation(
            sequence = entity.sequence,
            idempotencyKey = entity.idempotencyKey,
            groupId = entity.groupId,
            entityType = EntityType.valueOf(entity.entityType),
            entityId = entity.entityId,
            parentId = entity.parentId,
            operation = OperationType.valueOf(entity.operation),
            payload = entity.payload,
            createdAt = entity.createdAt,
            attempts = entity.attempts,
            lastError = entity.lastError,
        )
    }
}

/**
 * Generates negative ids for rows created while offline. Server ids are always
 * positive, so the two spaces cannot collide.
 */
object TempIds {
    private val last = AtomicLong(System.currentTimeMillis())

    fun next(): Long = -last.incrementAndGet()
}
