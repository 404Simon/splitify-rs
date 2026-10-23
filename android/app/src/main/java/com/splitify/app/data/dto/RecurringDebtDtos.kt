package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class RecurringDebtDto(
    val id: Long,
    @SerialName("group_id") val groupId: Long,
    @SerialName("created_by") val createdBy: Long,
    @SerialName("creator_username") val creatorUsername: String,
    val name: String,
    val amount: String,
    val frequency: String,
    @SerialName("start_date") val startDate: String,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("next_generation_date") val nextGenerationDate: String,
    @SerialName("is_active") val isActive: Boolean,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
    @SerialName("is_creator") val isCreator: Boolean,
    val status: String,
)

@Serializable
data class RecurringDebtMemberDto(
    @SerialName("user_id") val userId: Long,
    val username: String,
    @SerialName("share_amount") val shareAmount: String,
)

@Serializable
data class GeneratedInstanceDto(
    @SerialName("shared_debt_id") val sharedDebtId: Long,
    @SerialName("debt_name") val debtName: String,
    val amount: String,
    @SerialName("created_at") val createdAt: String,
)

@Serializable
data class CreateRecurringDebtRequest(
    val name: String,
    val amount: String,
    val frequency: String,
    @SerialName("start_date") val startDate: String,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("member_ids") val memberIds: List<Long>,
)

@Serializable
data class UpdateRecurringDebtRequest(
    val name: String,
    val amount: String,
    val frequency: String,
    @SerialName("end_date") val endDate: String? = null,
    @SerialName("is_active") val isActive: Boolean,
    @SerialName("member_ids") val memberIds: List<Long>,
)

@Serializable
data class ToggleRecurringResponse(
    @SerialName("is_active") val isActive: Boolean,
)

@Serializable
data class GenerateRecurringResponse(
    @SerialName("shared_debt_id") val sharedDebtId: Long,
)
