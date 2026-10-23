package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class ShoppingListSummaryDto(
    val id: Long,
    val name: String,
    @SerialName("group_id") val groupId: Long,
    @SerialName("created_by") val createdBy: Long,
    @SerialName("creator_username") val creatorUsername: String,
    @SerialName("total_items") val totalItems: Long,
    @SerialName("completed_items") val completedItems: Long,
)

@Serializable
data class ShoppingListDto(
    val id: Long,
    @SerialName("group_id") val groupId: Long,
    @SerialName("created_by") val createdBy: Long,
    val name: String,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class ShoppingListItemDto(
    val id: Long,
    @SerialName("shopping_list_id") val shoppingListId: Long,
    val name: String,
    val quantity: String? = null,
    val category: String? = null,
    @SerialName("is_completed") val isCompleted: Boolean,
    @SerialName("completed_by") val completedBy: Long? = null,
    @SerialName("completed_by_username") val completedByUsername: String? = null,
    @SerialName("completed_at") val completedAt: String? = null,
    val position: Long,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class ShoppingListActivityDto(
    val id: Long,
    @SerialName("shopping_list_id") val shoppingListId: Long,
    @SerialName("user_id") val userId: Long,
    val username: String,
    val action: String,
    @SerialName("item_name") val itemName: String,
    @SerialName("created_at") val createdAt: String,
)

@Serializable
data class CreateShoppingListRequest(
    val name: String,
)

@Serializable
data class CreateShoppingItemRequest(
    val name: String,
    val quantity: String? = null,
    val category: String? = null,
)

@Serializable
data class ToggleItemResponse(
    @SerialName("is_completed") val isCompleted: Boolean,
)
