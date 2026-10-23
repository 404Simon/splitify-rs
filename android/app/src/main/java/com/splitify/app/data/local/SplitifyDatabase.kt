package com.splitify.app.data.local

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import com.splitify.app.data.local.dao.IdMappingDao
import com.splitify.app.data.local.dao.MetadataDao
import com.splitify.app.data.local.dao.OutboxDao
import com.splitify.app.data.local.dao.SnapshotDao
import com.splitify.app.data.local.entity.GroupSnapshotEntity
import com.splitify.app.data.local.entity.GroupsCacheEntity
import com.splitify.app.data.local.entity.IdMappingEntity
import com.splitify.app.data.local.entity.LocalMetadataEntity
import com.splitify.app.data.local.entity.OutboxOperationEntity

/**
 * Local store for the offline cache and the pending-operation queue.
 *
 * The cached snapshots are disposable: bumping [VERSION] may drop them. The
 * outbox is intentionally included in the same database so a local mutation
 * and its pending operation are committed atomically.
 */
@Database(
    entities = [
        OutboxOperationEntity::class,
        IdMappingEntity::class,
        GroupSnapshotEntity::class,
        GroupsCacheEntity::class,
        LocalMetadataEntity::class,
    ],
    version = 2,
    exportSchema = false,
)
abstract class SplitifyDatabase : RoomDatabase() {

    abstract fun outboxDao(): OutboxDao

    abstract fun idMappingDao(): IdMappingDao

    abstract fun snapshotDao(): SnapshotDao

    abstract fun metadataDao(): MetadataDao

    companion object {
        fun build(context: Context): SplitifyDatabase =
            Room.databaseBuilder(
                context.applicationContext,
                SplitifyDatabase::class.java,
                "splitify-offline.db",
            )
                .fallbackToDestructiveMigration(true)
                .build()
    }
}
