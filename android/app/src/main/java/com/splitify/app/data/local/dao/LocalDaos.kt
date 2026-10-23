package com.splitify.app.data.local.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.splitify.app.data.local.entity.GroupSnapshotEntity
import com.splitify.app.data.local.entity.GroupsCacheEntity
import com.splitify.app.data.local.entity.IdMappingEntity
import com.splitify.app.data.local.entity.LocalMetadataEntity
import com.splitify.app.data.local.entity.OutboxOperationEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface OutboxDao {

    @Query("SELECT * FROM outbox_operations ORDER BY sequence ASC")
    suspend fun all(): List<OutboxOperationEntity>

    @Query("SELECT * FROM outbox_operations WHERE groupId = :groupId ORDER BY sequence ASC")
    suspend fun forGroup(groupId: Long): List<OutboxOperationEntity>

    @Query("SELECT COUNT(*) FROM outbox_operations")
    fun observeCount(): Flow<Int>

    @Query("SELECT COUNT(*) FROM outbox_operations WHERE groupId = :groupId")
    suspend fun countForGroup(groupId: Long): Int

    @Insert
    suspend fun insert(operation: OutboxOperationEntity)

    @Query("UPDATE outbox_operations SET attempts = attempts + 1, lastError = :error WHERE sequence = :sequence")
    suspend fun recordFailure(sequence: Long, error: String)

    @Query("DELETE FROM outbox_operations WHERE sequence = :sequence")
    suspend fun delete(sequence: Long)

    @Query("DELETE FROM outbox_operations WHERE entityType = :entityType AND entityId = :entityId")
    suspend fun deleteForEntity(entityType: String, entityId: Long)

    @Query("DELETE FROM outbox_operations WHERE groupId = :groupId")
    suspend fun deleteForGroup(groupId: Long)

    @Query("DELETE FROM outbox_operations")
    suspend fun deleteAll()

    @Query("UPDATE outbox_operations SET groupId = :serverId WHERE groupId = :localId")
    suspend fun reassignGroup(localId: Long, serverId: Long)
}

@Dao
interface IdMappingDao {

    @Query("SELECT serverId FROM id_mappings WHERE mappingKey = :mappingKey")
    suspend fun serverId(mappingKey: String): Long?

    @Query("SELECT * FROM id_mappings")
    suspend fun all(): List<IdMappingEntity>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun put(mapping: IdMappingEntity)

    @Query("DELETE FROM id_mappings")
    suspend fun deleteAll()
}

@Dao
interface SnapshotDao {

    @Query("SELECT * FROM group_snapshots WHERE groupId = :groupId")
    suspend fun get(groupId: Long): GroupSnapshotEntity?

    @Query("SELECT * FROM group_snapshots")
    suspend fun all(): List<GroupSnapshotEntity>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun put(snapshot: GroupSnapshotEntity)

    @Query("DELETE FROM group_snapshots WHERE groupId = :groupId")
    suspend fun delete(groupId: Long)

    @Query("SELECT * FROM groups_cache WHERE id = 0")
    suspend fun getGroups(): GroupsCacheEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun putGroups(groups: GroupsCacheEntity)

    @Query("DELETE FROM group_snapshots")
    suspend fun deleteAll()

    @Query("DELETE FROM groups_cache")
    suspend fun deleteGroups()
}

@Dao
interface MetadataDao {

    @Query("SELECT value FROM local_metadata WHERE key = :key")
    suspend fun get(key: String): String?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun put(metadata: LocalMetadataEntity)

    @Query("DELETE FROM local_metadata")
    suspend fun clear()
}
