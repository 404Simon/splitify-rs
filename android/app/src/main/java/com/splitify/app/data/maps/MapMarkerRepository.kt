package com.splitify.app.data.maps

import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.cache.EntityType
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.cache.OperationType
import com.splitify.app.data.cache.PendingOperation
import com.splitify.app.data.cache.TempIds
import com.splitify.app.data.dto.CreateMarkerRequest
import com.splitify.app.data.dto.GroupSnapshotDto
import com.splitify.app.data.dto.MapMarkerDto
import com.splitify.app.data.network.AppJson
import com.splitify.app.data.sync.SyncManager
import java.time.Instant
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class MapMarkerRepository(
    private val localStore: LocalStore,
    private val syncManager: SyncManager,
    private val session: SessionManager,
) {

    fun observeMarkers(groupId: Long): Flow<List<MapMarkerDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.markers.orEmpty()
                .sortedWith(
                    compareByDescending<MapMarkerDto> { it.createdAt }.thenByDescending { it.id },
                )
        }

    suspend fun createMarker(
        groupId: Long,
        request: CreateMarkerRequest,
    ): Result<Long> = runCatching {
        val user = session.requireUser()
        val id = TempIds.next()
        val now = Instant.now().toString()
        val marker = MapMarkerDto(
            id = id,
            groupId = groupId,
            createdBy = user.id,
            creatorUsername = user.username,
            name = request.name,
            description = request.description,
            address = request.address,
            emoji = request.emoji,
            latitude = request.latitude,
            longitude = request.longitude,
            createdAt = now,
            updatedAt = now,
        )
        enqueue(
            groupId = groupId,
            entityId = id,
            operation = OperationType.CREATE,
            payload = AppJson.encodeToString(CreateMarkerRequest.serializer(), request),
            transform = { current -> current.copy(markers = listOf(marker) + current.markers) },
        )
        id
    }

    suspend fun updateMarker(
        groupId: Long,
        markerId: Long,
        request: CreateMarkerRequest,
    ): Result<Unit> = runCatching {
        val snapshot = localStore.groupSnapshot(groupId)
            ?: error("Group $groupId is not cached yet")
        val effectiveId = snapshot.markers.firstOrNull { it.id == markerId }?.id
            ?: localStore.currentId(EntityType.MARKER, markerId)
        enqueue(
            groupId = groupId,
            entityId = effectiveId,
            operation = OperationType.UPDATE,
            payload = AppJson.encodeToString(CreateMarkerRequest.serializer(), request),
            transform = { current ->
                current.copy(
                    markers = current.markers.map { marker ->
                        if (marker.id != effectiveId) {
                            marker
                        } else {
                            marker.copy(
                                name = request.name,
                                description = request.description,
                                address = request.address,
                                emoji = request.emoji,
                                latitude = request.latitude,
                                longitude = request.longitude,
                                updatedAt = Instant.now().toString(),
                            )
                        }
                    },
                )
            },
        )
    }

    suspend fun deleteMarker(groupId: Long, markerId: Long): Result<Unit> = runCatching {
        val snapshot = localStore.groupSnapshot(groupId)
            ?: error("Group $groupId is not cached yet")
        val effectiveId = snapshot.markers.firstOrNull { it.id == markerId }?.id
            ?: localStore.currentId(EntityType.MARKER, markerId)
        enqueue(
            groupId = groupId,
            entityId = effectiveId,
            operation = OperationType.DELETE,
            transform = { current ->
                current.copy(markers = current.markers.filterNot { it.id == effectiveId })
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
                entityType = EntityType.MARKER,
                entityId = entityId,
                operation = operation,
                payload = payload,
            ),
            transform = transform,
        )
        syncManager.requestSync()
    }
}
