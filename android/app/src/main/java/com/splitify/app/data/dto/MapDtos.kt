package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class MapMarkerDto(
    val id: Long,
    @SerialName("group_id") val groupId: Long,
    @SerialName("created_by") val createdBy: Long,
    @SerialName("creator_username") val creatorUsername: String,
    val name: String,
    val description: String? = null,
    val address: String? = null,
    val emoji: String,
    val latitude: Double,
    val longitude: Double,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class CreateMarkerRequest(
    val name: String,
    val description: String? = null,
    val address: String? = null,
    val emoji: String = "📍",
    val latitude: Double,
    val longitude: Double,
)
