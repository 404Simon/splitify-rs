package com.splitify.app.data.maps

import com.splitify.app.data.dto.CreateMarkerRequest
import com.splitify.app.data.dto.IdResponse
import com.splitify.app.data.dto.MapMarkerDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.serialization.builtins.ListSerializer

class MapMarkerRepository(private val api: ApiClient) {

    suspend fun listMarkers(groupId: Long): Result<List<MapMarkerDto>> = apiResult {
        api.get("groups/$groupId/markers", ListSerializer(MapMarkerDto.serializer()))
    }

    suspend fun createMarker(
        groupId: Long,
        request: CreateMarkerRequest,
    ): Result<Long> = apiResult {
        api.post(
            path = "groups/$groupId/markers",
            body = request,
            bodySerializer = CreateMarkerRequest.serializer(),
            serializer = IdResponse.serializer(),
        ).id
    }

    suspend fun updateMarker(
        markerId: Long,
        request: CreateMarkerRequest,
    ): Result<Unit> = apiResult {
        api.patch(
            path = "markers/$markerId",
            body = request,
            bodySerializer = CreateMarkerRequest.serializer(),
        )
    }

    suspend fun deleteMarker(markerId: Long): Result<Unit> = apiResult {
        api.delete("markers/$markerId")
    }
}
