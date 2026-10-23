package com.splitify.app.data.local.entity

import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

/**
 * A mutation that has been applied locally and still has to reach the server.
 *
 * Operations are pushed in [sequence] order. [idempotencyKey] is sent with the
 * request so the server can safely replay it after a lost response.
 */
@Entity(
    tableName = "outbox_operations",
    indices = [Index("groupId"), Index(value = ["entityType", "entityId"])],
)
data class OutboxOperationEntity(
    @PrimaryKey(autoGenerate = true) val sequence: Long = 0,
    val idempotencyKey: String,
    val groupId: Long,
    val entityType: String,
    val entityId: Long,
    val parentId: Long? = null,
    val operation: String,
    /** JSON request body, encoded with the same Json instance as the API client. */
    val payload: String = "{}",
    val createdAt: Long,
    val attempts: Int = 0,
    val lastError: String? = null,
)

/** Maps a locally generated (negative) id to the id assigned by the server. */
@Entity(tableName = "id_mappings")
data class IdMappingEntity(
    @PrimaryKey val mappingKey: String,
    val serverId: Long,
)

/** Last successful snapshot for a group. */
@Entity(tableName = "group_snapshots")
data class GroupSnapshotEntity(
    @PrimaryKey val groupId: Long,
    val json: String,
    val syncedAt: Long,
)

/** Cached result of `GET /groups` (a single row with id 0). */
@Entity(tableName = "groups_cache")
data class GroupsCacheEntity(
    @PrimaryKey val id: Int = 0,
    val json: String,
    val syncedAt: Long,
)

/** Small key/value store for cache metadata (currently the owning user id). */
@Entity(tableName = "local_metadata")
data class LocalMetadataEntity(
    @PrimaryKey val key: String,
    val value: String,
)
