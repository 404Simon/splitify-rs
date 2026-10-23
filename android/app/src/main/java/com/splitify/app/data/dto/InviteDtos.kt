package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class InviteListItemDto(
    val uuid: String,
    val name: String? = null,
    @SerialName("is_reusable") val isReusable: Boolean,
    @SerialName("expiration_date") val expirationDate: String,
)

@Serializable
data class InvitePreviewDto(
    val uuid: String,
    @SerialName("group_id") val groupId: Long,
    @SerialName("group_name") val groupName: String,
    val name: String? = null,
    @SerialName("is_reusable") val isReusable: Boolean,
    @SerialName("duration_days") val durationDays: Long,
    @SerialName("created_at") val createdAt: String,
    @SerialName("is_valid") val isValid: Boolean,
    @SerialName("expires_at") val expiresAt: String,
)

@Serializable
data class CreateInviteRequest(
    val name: String? = null,
    @SerialName("is_reusable") val isReusable: Boolean,
    @SerialName("duration_days") val durationDays: Long,
)

@Serializable
data class InviteCreatedDto(
    val uuid: String,
)

@Serializable
data class AcceptInviteResponse(
    @SerialName("group_id") val groupId: Long,
)
