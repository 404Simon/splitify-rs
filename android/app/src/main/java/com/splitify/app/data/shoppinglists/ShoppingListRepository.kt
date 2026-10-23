package com.splitify.app.data.shoppinglists

import com.splitify.app.data.dto.CreateShoppingItemRequest
import com.splitify.app.data.dto.CreateShoppingListRequest
import com.splitify.app.data.dto.IdResponse
import com.splitify.app.data.dto.ShoppingListActivityDto
import com.splitify.app.data.dto.ShoppingListDto
import com.splitify.app.data.dto.ShoppingListItemDto
import com.splitify.app.data.dto.ShoppingListSummaryDto
import com.splitify.app.data.dto.ToggleItemResponse
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.serialization.builtins.ListSerializer

class ShoppingListRepository(private val api: ApiClient) {

    suspend fun listLists(groupId: Long): Result<List<ShoppingListSummaryDto>> = apiResult {
        api.get("groups/$groupId/shopping-lists", ListSerializer(ShoppingListSummaryDto.serializer()))
    }

    suspend fun createList(groupId: Long, name: String): Result<Long> = apiResult {
        api.post(
            path = "groups/$groupId/shopping-lists",
            body = CreateShoppingListRequest(name = name),
            bodySerializer = CreateShoppingListRequest.serializer(),
            serializer = IdResponse.serializer(),
        ).id
    }

    suspend fun getList(listId: Long): Result<ShoppingListDto> = apiResult {
        api.get("shopping-lists/$listId", ShoppingListDto.serializer())
    }

    suspend fun updateList(listId: Long, name: String): Result<Unit> = apiResult {
        api.patch(
            path = "shopping-lists/$listId",
            body = CreateShoppingListRequest(name = name),
            bodySerializer = CreateShoppingListRequest.serializer(),
        )
    }

    suspend fun deleteList(listId: Long): Result<Unit> = apiResult {
        api.delete("shopping-lists/$listId")
    }

    suspend fun listItems(listId: Long): Result<List<ShoppingListItemDto>> = apiResult {
        api.get("shopping-lists/$listId/items", ListSerializer(ShoppingListItemDto.serializer()))
    }

    suspend fun addItem(
        listId: Long,
        name: String,
        quantity: String?,
        category: String?,
    ): Result<Long> = apiResult {
        api.post(
            path = "shopping-lists/$listId/items",
            body = CreateShoppingItemRequest(
                name = name,
                quantity = quantity?.takeIf { it.isNotBlank() },
                category = category?.takeIf { it.isNotBlank() },
            ),
            bodySerializer = CreateShoppingItemRequest.serializer(),
            serializer = IdResponse.serializer(),
        ).id
    }

    suspend fun updateItem(
        listId: Long,
        itemId: Long,
        name: String,
        quantity: String?,
        category: String?,
    ): Result<Unit> = apiResult {
        api.patch(
            path = "shopping-lists/$listId/items/$itemId",
            body = CreateShoppingItemRequest(
                name = name,
                quantity = quantity?.takeIf { it.isNotBlank() },
                category = category?.takeIf { it.isNotBlank() },
            ),
            bodySerializer = CreateShoppingItemRequest.serializer(),
        )
    }

    suspend fun toggleItem(listId: Long, itemId: Long): Result<Boolean> = apiResult {
        api.post(
            path = "shopping-lists/$listId/items/$itemId/toggle",
            body = kotlinx.serialization.json.JsonObject(emptyMap()),
            bodySerializer = kotlinx.serialization.json.JsonObject.serializer(),
            serializer = ToggleItemResponse.serializer(),
        ).isCompleted
    }

    suspend fun deleteItem(listId: Long, itemId: Long): Result<Unit> = apiResult {
        api.delete("shopping-lists/$listId/items/$itemId")
    }

    suspend fun listActivity(listId: Long): Result<List<ShoppingListActivityDto>> = apiResult {
        api.get(
            "shopping-lists/$listId/activity",
            ListSerializer(ShoppingListActivityDto.serializer()),
        )
    }
}
