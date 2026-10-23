package com.splitify.app.data.network

import com.splitify.app.data.dto.AcceptInviteResponse
import com.splitify.app.data.dto.CreateDebtRequest
import com.splitify.app.data.dto.CreateGroupRequest
import com.splitify.app.data.dto.CreateInviteRequest
import com.splitify.app.data.dto.CreateMarkerRequest
import com.splitify.app.data.dto.CreateRecurringDebtRequest
import com.splitify.app.data.dto.CreateShoppingItemRequest
import com.splitify.app.data.dto.CreateShoppingListRequest
import com.splitify.app.data.dto.CreateTransactionRequest
import com.splitify.app.data.dto.GenerateRecurringResponse
import com.splitify.app.data.dto.GroupSnapshotDto
import com.splitify.app.data.dto.GroupSummaryDto
import com.splitify.app.data.dto.IdResponse
import com.splitify.app.data.dto.InviteCreatedDto
import com.splitify.app.data.dto.ToggleItemRequest
import com.splitify.app.data.dto.ToggleItemResponse
import com.splitify.app.data.dto.ToggleRecurringRequest
import com.splitify.app.data.dto.ToggleRecurringResponse
import com.splitify.app.data.dto.UpdateGroupRequest
import com.splitify.app.data.dto.UpdateRecurringDebtRequest
import kotlinx.serialization.builtins.ListSerializer

/**
 * Thin typed wrapper around every endpoint used by the sync engine.
 *
 * All writes accept an idempotency key so a retried outbox operation cannot
 * create a duplicate row when the first response was lost in transit.
 */
class SyncApi(private val api: ApiClient) {

    suspend fun listGroups(): List<GroupSummaryDto> =
        api.get("groups", ListSerializer(GroupSummaryDto.serializer()))

    suspend fun groupSnapshot(groupId: Long): GroupSnapshotDto =
        api.get("groups/$groupId/snapshot", GroupSnapshotDto.serializer())

    suspend fun createGroup(name: String, key: String): Long =
        api.post(
            path = "groups",
            body = CreateGroupRequest(name),
            bodySerializer = CreateGroupRequest.serializer(),
            serializer = IdResponse.serializer(),
            idempotencyKey = key,
        ).id

    suspend fun updateGroup(id: Long, body: UpdateGroupRequest, key: String) =
        api.patch(
            "groups/$id",
            body,
            UpdateGroupRequest.serializer(),
            idempotencyKey = key,
        )

    suspend fun deleteGroup(id: Long, key: String) =
        api.delete("groups/$id", idempotencyKey = key)

    suspend fun createDebt(groupId: Long, body: CreateDebtRequest, key: String): Long =
        api.post(
            path = "groups/$groupId/debts",
            body = body,
            bodySerializer = CreateDebtRequest.serializer(),
            serializer = IdResponse.serializer(),
            idempotencyKey = key,
        ).id

    suspend fun updateDebt(id: Long, body: CreateDebtRequest, key: String) =
        api.patch("debts/$id", body, CreateDebtRequest.serializer(), idempotencyKey = key)

    suspend fun deleteDebt(id: Long, key: String) =
        api.delete("debts/$id", idempotencyKey = key)

    suspend fun createTransaction(
        groupId: Long,
        body: CreateTransactionRequest,
        key: String,
    ): Long = api.post(
        path = "groups/$groupId/transactions",
        body = body,
        bodySerializer = CreateTransactionRequest.serializer(),
        serializer = IdResponse.serializer(),
        idempotencyKey = key,
    ).id

    suspend fun updateTransaction(id: Long, body: CreateTransactionRequest, key: String) =
        api.patch("transactions/$id", body, CreateTransactionRequest.serializer(), idempotencyKey = key)

    suspend fun deleteTransaction(id: Long, key: String) =
        api.delete("transactions/$id", idempotencyKey = key)

    suspend fun createShoppingList(groupId: Long, body: CreateShoppingListRequest, key: String): Long =
        api.post(
            path = "groups/$groupId/shopping-lists",
            body = body,
            bodySerializer = CreateShoppingListRequest.serializer(),
            serializer = IdResponse.serializer(),
            idempotencyKey = key,
        ).id

    suspend fun updateShoppingList(id: Long, body: CreateShoppingListRequest, key: String) =
        api.patch("shopping-lists/$id", body, CreateShoppingListRequest.serializer(), idempotencyKey = key)

    suspend fun deleteShoppingList(id: Long, key: String) =
        api.delete("shopping-lists/$id", idempotencyKey = key)

    suspend fun createShoppingItem(
        listId: Long,
        body: CreateShoppingItemRequest,
        key: String,
    ): Long = api.post(
        path = "shopping-lists/$listId/items",
        body = body,
        bodySerializer = CreateShoppingItemRequest.serializer(),
        serializer = IdResponse.serializer(),
        idempotencyKey = key,
    ).id

    suspend fun updateShoppingItem(
        listId: Long,
        itemId: Long,
        body: CreateShoppingItemRequest,
        key: String,
    ) = api.patch(
        "shopping-lists/$listId/items/$itemId",
        body,
        CreateShoppingItemRequest.serializer(),
        idempotencyKey = key,
    )

    suspend fun deleteShoppingItem(listId: Long, itemId: Long, key: String) =
        api.delete("shopping-lists/$listId/items/$itemId", idempotencyKey = key)

    suspend fun toggleShoppingItem(listId: Long, itemId: Long, desired: Boolean, key: String) {
        api.post(
            path = "shopping-lists/$listId/items/$itemId/toggle",
            body = ToggleItemRequest(desired),
            bodySerializer = ToggleItemRequest.serializer(),
            serializer = ToggleItemResponse.serializer(),
            idempotencyKey = key,
        )
    }

    suspend fun createRecurringDebt(
        groupId: Long,
        body: CreateRecurringDebtRequest,
        key: String,
    ): Long = api.post(
        path = "groups/$groupId/recurring-debts",
        body = body,
        bodySerializer = CreateRecurringDebtRequest.serializer(),
        serializer = IdResponse.serializer(),
        idempotencyKey = key,
    ).id

    suspend fun updateRecurringDebt(id: Long, body: UpdateRecurringDebtRequest, key: String) =
        api.patch("recurring-debts/$id", body, UpdateRecurringDebtRequest.serializer(), idempotencyKey = key)

    suspend fun deleteRecurringDebt(id: Long, key: String) =
        api.delete("recurring-debts/$id", idempotencyKey = key)

    suspend fun toggleRecurringDebt(id: Long, desired: Boolean, key: String) {
        api.post(
            path = "recurring-debts/$id/toggle",
            body = ToggleRecurringRequest(desired),
            bodySerializer = ToggleRecurringRequest.serializer(),
            serializer = ToggleRecurringResponse.serializer(),
            idempotencyKey = key,
        )
    }

    suspend fun generateRecurringDebt(id: Long, key: String) {
        api.post(
            path = "recurring-debts/$id/generate",
            body = kotlinx.serialization.json.JsonObject(emptyMap()),
            bodySerializer = kotlinx.serialization.json.JsonObject.serializer(),
            serializer = GenerateRecurringResponse.serializer(),
            idempotencyKey = key,
        )
    }

    suspend fun createMarker(groupId: Long, body: CreateMarkerRequest, key: String): Long =
        api.post(
            path = "groups/$groupId/markers",
            body = body,
            bodySerializer = CreateMarkerRequest.serializer(),
            serializer = IdResponse.serializer(),
            idempotencyKey = key,
        ).id

    suspend fun updateMarker(id: Long, body: CreateMarkerRequest, key: String) =
        api.patch("markers/$id", body, CreateMarkerRequest.serializer(), idempotencyKey = key)

    suspend fun deleteMarker(id: Long, key: String) =
        api.delete("markers/$id", idempotencyKey = key)

    suspend fun createInvite(
        groupId: Long,
        body: CreateInviteRequest,
        key: String,
    ): String = api.post(
        path = "groups/$groupId/invites",
        body = body,
        bodySerializer = CreateInviteRequest.serializer(),
        serializer = InviteCreatedDto.serializer(),
        idempotencyKey = key,
    ).uuid

    suspend fun deleteInvite(groupId: Long, uuid: String, key: String) =
        api.delete("groups/$groupId/invites/$uuid", idempotencyKey = key)

    suspend fun acceptInvite(uuid: String): Long = api.post(
        path = "invites/$uuid/accept",
        body = kotlinx.serialization.json.JsonObject(emptyMap()),
        bodySerializer = kotlinx.serialization.json.JsonObject.serializer(),
        serializer = AcceptInviteResponse.serializer(),
    ).groupId
}
