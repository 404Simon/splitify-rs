package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class SharedDebtDto(
    val id: Long,
    @SerialName("group_id") val groupId: Long,
    @SerialName("created_by") val createdBy: Long,
    @SerialName("creator_username") val creatorUsername: String,
    val name: String,
    val amount: String,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
    @SerialName("is_creator") val isCreator: Boolean,
)

@Serializable
data class UserShareDto(
    @SerialName("user_id") val userId: Long,
    val username: String,
    @SerialName("share_amount") val shareAmount: String,
)

@Serializable
data class CreateDebtRequest(
    val name: String,
    val amount: String,
    @SerialName("member_ids") val memberIds: List<Long>,
)

@Serializable
data class IdResponse(
    val id: Long,
)
