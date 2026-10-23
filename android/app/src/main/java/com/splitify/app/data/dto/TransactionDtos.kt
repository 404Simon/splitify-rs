package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class TransactionDto(
    val id: Long,
    @SerialName("group_id") val groupId: Long,
    @SerialName("payer_id") val payerId: Long,
    @SerialName("payer_username") val payerUsername: String,
    @SerialName("recipient_id") val recipientId: Long,
    @SerialName("recipient_username") val recipientUsername: String,
    val amount: String,
    val description: String? = null,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class DebtRelationshipDto(
    @SerialName("other_user_id") val otherUserId: Long,
    @SerialName("other_username") val otherUsername: String,
    val amount: String,
    @SerialName("relationship_type") val relationshipType: String,
)

@Serializable
data class UserBalanceDto(
    @SerialName("user_id") val userId: Long,
    val username: String,
    val relationships: List<DebtRelationshipDto> = emptyList(),
    @SerialName("total_owed") val totalOwed: String,
    @SerialName("total_owing") val totalOwing: String,
    @SerialName("net_amount") val netAmount: String,
    @SerialName("net_type") val netType: String,
)

@Serializable
data class CreateTransactionRequest(
    @SerialName("recipient_id") val recipientId: Long,
    val amount: String,
    val description: String? = null,
)
