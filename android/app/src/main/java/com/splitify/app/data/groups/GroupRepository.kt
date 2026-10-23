package com.splitify.app.data.groups

import com.splitify.app.data.dto.CreateGroupRequest
import com.splitify.app.data.dto.CreateGroupResponse
import com.splitify.app.data.dto.GroupDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.GroupSummaryDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.serialization.builtins.ListSerializer

class GroupRepository(private val api: ApiClient) {

    suspend fun listGroups(): Result<List<GroupSummaryDto>> = apiResult {
        api.get("groups", ListSerializer(GroupSummaryDto.serializer()))
    }

    suspend fun createGroup(name: String): Result<Long> = apiResult {
        api.post(
            path = "groups",
            body = CreateGroupRequest(name = name),
            bodySerializer = CreateGroupRequest.serializer(),
            serializer = CreateGroupResponse.serializer(),
        ).id
    }

    suspend fun getGroup(groupId: Long): Result<GroupDto> = apiResult {
        api.get("groups/$groupId", GroupDto.serializer())
    }

    suspend fun getMembers(groupId: Long): Result<List<GroupMemberDto>> = apiResult {
        api.get("groups/$groupId/members", ListSerializer(GroupMemberDto.serializer()))
    }
}
