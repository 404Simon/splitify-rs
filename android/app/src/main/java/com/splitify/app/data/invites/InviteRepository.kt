package com.splitify.app.data.invites

import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.config.ServerConfig
import com.splitify.app.data.config.ServerUrl
import com.splitify.app.data.dto.AcceptInviteResponse
import com.splitify.app.data.dto.CreateInviteRequest
import com.splitify.app.data.dto.InviteCreatedDto
import com.splitify.app.data.dto.InviteListItemDto
import com.splitify.app.data.dto.InvitePreviewDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import com.splitify.app.data.sync.SyncManager
import java.util.UUID
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.serialization.json.JsonObject

class InviteRepository(
    private val api: ApiClient,
    private val localStore: LocalStore,
    private val syncManager: SyncManager,
    private val serverConfig: ServerConfig,
) {

    fun observeInvites(groupId: Long): Flow<List<InviteListItemDto>> =
        localStore.observeGroup(groupId).map { it?.invites.orEmpty() }

    /**
     * Invites are creator-only on the server, so the UI needs to know who owns
     * the group without asking the network.
     */
    fun observeCreatorId(groupId: Long): Flow<Long?> =
        localStore.observeGroup(groupId).map { it?.group?.createdBy }

    suspend fun createInvite(
        groupId: Long,
        name: String?,
        isReusable: Boolean,
        durationDays: Long,
    ): Result<String> = apiResult {
        val uuid = api.post(
            path = "groups/$groupId/invites",
            body = CreateInviteRequest(
                name = name?.takeIf { it.isNotBlank() },
                isReusable = isReusable,
                durationDays = durationDays,
            ),
            bodySerializer = CreateInviteRequest.serializer(),
            serializer = InviteCreatedDto.serializer(),
            idempotencyKey = UUID.randomUUID().toString(),
        ).uuid
        syncManager.requestSync()
        uuid
    }

    suspend fun deleteInvite(groupId: Long, uuid: String): Result<Unit> = apiResult {
        api.delete(
            path = "groups/$groupId/invites/$uuid",
            idempotencyKey = UUID.randomUUID().toString(),
        )
        syncManager.requestSync()
    }

    suspend fun previewInvite(uuid: String): Result<InvitePreviewDto> = apiResult {
        api.get(
            path = "invites/$uuid",
            serializer = InvitePreviewDto.serializer(),
            authenticated = false,
        )
    }

    suspend fun acceptInvite(uuid: String): Result<Long> = apiResult {
        val groupId = api.post(
            path = "invites/$uuid/accept",
            body = JsonObject(emptyMap()),
            bodySerializer = JsonObject.serializer(),
            serializer = AcceptInviteResponse.serializer(),
            idempotencyKey = UUID.randomUUID().toString(),
        ).groupId
        syncManager.syncGroup(groupId)
        syncManager.requestSync()
        groupId
    }

    /** Null when no server is configured, i.e. there is no origin to link to. */
    fun inviteLink(uuid: String): String? {
        val origin = serverConfig.baseUrl.value
            ?.trimEnd('/')
            ?.removeSuffix(ServerUrl.API_PREFIX)
            ?.takeIf { it.isNotBlank() }
            ?: return null
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
