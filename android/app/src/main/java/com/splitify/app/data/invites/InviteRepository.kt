package com.splitify.app.data.invites

import com.splitify.app.BuildConfig
import com.splitify.app.data.dto.AcceptInviteResponse
import com.splitify.app.data.dto.CreateInviteRequest
import com.splitify.app.data.dto.InviteCreatedDto
import com.splitify.app.data.dto.InviteListItemDto
import com.splitify.app.data.dto.InvitePreviewDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.JsonObject

class InviteRepository(private val api: ApiClient) {

    suspend fun listGroupInvites(groupId: Long): Result<List<InviteListItemDto>> = apiResult {
        api.get("groups/$groupId/invites", ListSerializer(InviteListItemDto.serializer()))
    }

    suspend fun createInvite(
        groupId: Long,
        name: String?,
        isReusable: Boolean,
        durationDays: Long,
    ): Result<String> = apiResult {
        api.post(
            path = "groups/$groupId/invites",
            body = CreateInviteRequest(
                name = name?.takeIf { it.isNotBlank() },
                isReusable = isReusable,
                durationDays = durationDays,
            ),
            bodySerializer = CreateInviteRequest.serializer(),
            serializer = InviteCreatedDto.serializer(),
        ).uuid
    }

    suspend fun deleteInvite(groupId: Long, uuid: String): Result<Unit> = apiResult {
        api.delete("groups/$groupId/invites/$uuid")
    }

    suspend fun previewInvite(uuid: String): Result<InvitePreviewDto> = apiResult {
        api.get(
            path = "invites/$uuid",
            serializer = InvitePreviewDto.serializer(),
            authenticated = false,
        )
    }

    suspend fun acceptInvite(uuid: String): Result<Long> = apiResult {
        api.post(
            path = "invites/$uuid/accept",
            body = JsonObject(emptyMap()),
            bodySerializer = JsonObject.serializer(),
            serializer = AcceptInviteResponse.serializer(),
        ).groupId
    }

    fun inviteLink(uuid: String): String {
        val origin = BuildConfig.API_BASE_URL
            .trimEnd('/')
            .removeSuffix("/api/v1")
        return "$origin/invite/${uuid.trim()}"
    }

    fun extractInviteUuid(input: String): String? {
        val trimmed = input.trim()
        if (trimmed.isEmpty()) return null
        val withoutFragment = trimmed.substringBefore('#')
        val withoutQuery = withoutFragment.substringBefore('?')
        val segment = withoutQuery.trimEnd('/').substringAfterLast('/')
        return segment.takeIf { it.isNotBlank() }
    }
}
