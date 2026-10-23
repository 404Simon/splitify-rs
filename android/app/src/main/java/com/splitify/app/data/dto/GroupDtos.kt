package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class GroupSummaryDto(
    val id: Long,
    val name: String,
    @SerialName("created_by") val createdBy: Long,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
    @SerialName("member_count") val memberCount: Long,
    @SerialName("is_admin") val isAdmin: Boolean,
)

@Serializable
data class GroupDto(
    val id: Long,
    val name: String,
    @SerialName("created_by") val createdBy: Long,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class GroupMemberDto(
    val id: Long,
    val username: String,
    @SerialName("is_creator") val isCreator: Boolean,
)

@Serializable
data class CreateGroupRequest(
    val name: String,
)

@Serializable
data class CreateGroupResponse(
    val id: Long,
)

@Serializable
data class UpdateGroupRequest(
    val name: String,
    @SerialName("member_ids") val memberIds: List<Long>,
)
