package com.splitify.app.data.shoppinglists

import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.cache.EntityType
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.cache.OperationType
import com.splitify.app.data.cache.PendingOperation
import com.splitify.app.data.cache.TempIds
import com.splitify.app.data.dto.CreateShoppingItemRequest
import com.splitify.app.data.dto.CreateShoppingListRequest
import com.splitify.app.data.dto.GroupSnapshotDto
import com.splitify.app.data.dto.ShoppingListActivityDto
import com.splitify.app.data.dto.ShoppingListDto
import com.splitify.app.data.dto.ShoppingListItemDto
import com.splitify.app.data.dto.ShoppingListSnapshotDto
import com.splitify.app.data.dto.ShoppingListSummaryDto
import com.splitify.app.data.dto.ToggleItemRequest
import com.splitify.app.data.network.AppJson
import com.splitify.app.data.sync.SyncManager
import java.time.Instant
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class ShoppingListRepository(
    private val localStore: LocalStore,
    private val syncManager: SyncManager,
    private val session: SessionManager,
) {

    fun observeLists(groupId: Long): Flow<List<ShoppingListSummaryDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.let { group ->
                group.shoppingLists
                    .sortedByDescending { it.list.createdAt }
                    .map { summary(it, group) }
            }.orEmpty()
        }

    fun observeList(groupId: Long, listId: Long): Flow<ShoppingListDto?> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.listEntry(listId)?.list
        }

    fun observeItems(groupId: Long, listId: Long): Flow<List<ShoppingListItemDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.listEntry(listId)?.items.orEmpty()
                .sortedWith(
                    compareBy<ShoppingListItemDto> { it.position }.thenBy { it.createdAt },
                )
        }

    fun observeActivity(groupId: Long, listId: Long): Flow<List<ShoppingListActivityDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.listEntry(listId)?.activity.orEmpty()
                .sortedByDescending { it.createdAt }
                .take(ACTIVITY_LIMIT)
        }

    suspend fun createList(groupId: Long, name: String): Result<Long> = runCatching {
        val user = session.requireUser()
        val id = TempIds.next()
        val now = Instant.now().toString()
        val list = ShoppingListDto(
            id = id,
            groupId = groupId,
            createdBy = user.id,
            name = name,
            createdAt = now,
            updatedAt = now,
        )
        enqueue(
            groupId = groupId,
            entityType = EntityType.SHOPPING_LIST,
            entityId = id,
            operation = OperationType.CREATE,
            payload = AppJson.encodeToString(
                CreateShoppingListRequest.serializer(),
                CreateShoppingListRequest(name),
            ),
            transform = { current ->
                current.copy(
                    shoppingLists = current.shoppingLists + ShoppingListSnapshotDto(list = list),
                )
            },
        )
        id
    }

    suspend fun updateList(groupId: Long, listId: Long, name: String): Result<Unit> = runCatching {
        val effectiveListId = requireSnapshot(groupId).listEntry(listId)?.list?.id
            ?: localStore.currentId(EntityType.SHOPPING_LIST, listId)
        enqueue(
            groupId = groupId,
            entityType = EntityType.SHOPPING_LIST,
            entityId = effectiveListId,
            operation = OperationType.UPDATE,
            payload = AppJson.encodeToString(
                CreateShoppingListRequest.serializer(),
                CreateShoppingListRequest(name),
            ),
            transform = { current ->
                current.copy(
                    shoppingLists = current.shoppingLists.map { snapshot ->
                        if (snapshot.list.id != effectiveListId) {
                            snapshot
                        } else {
                            snapshot.copy(
                                list = snapshot.list.copy(
                                    name = name,
                                    updatedAt = Instant.now().toString(),
                                ),
                            )
                        }
                    },
                )
            },
        )
    }

    suspend fun deleteList(groupId: Long, listId: Long): Result<Unit> = runCatching {
        val effectiveListId = requireSnapshot(groupId).listEntry(listId)?.list?.id
            ?: localStore.currentId(EntityType.SHOPPING_LIST, listId)
        enqueue(
            groupId = groupId,
            entityType = EntityType.SHOPPING_LIST,
            entityId = effectiveListId,
            operation = OperationType.DELETE,
            transform = { current ->
                current.copy(
                    shoppingLists = current.shoppingLists.filterNot {
                        it.list.id == effectiveListId
                    },
                )
            },
        )
    }

    suspend fun addItem(
        groupId: Long,
        listId: Long,
        name: String,
        quantity: String?,
        category: String?,
    ): Result<Long> = runCatching {
        val user = session.requireUser()
        val id = TempIds.next()
        val now = Instant.now().toString()
        val snapshot = requireSnapshot(groupId)
        val existing = snapshot.listEntry(listId)
            ?: error("Shopping list $listId is not cached")
        val effectiveListId = existing.list.id
        val position = (existing.items.maxOfOrNull { it.position } ?: 0L) + 1
        val item = ShoppingListItemDto(
            id = id,
            shoppingListId = effectiveListId,
            name = name,
            quantity = quantity,
            category = category,
            isCompleted = false,
            completedBy = null,
            completedByUsername = null,
            completedAt = null,
            position = position,
            createdAt = now,
            updatedAt = now,
        )
        val request = CreateShoppingItemRequest(name, quantity, category)
        enqueue(
            groupId = groupId,
            entityType = EntityType.SHOPPING_ITEM,
            entityId = id,
            parentId = effectiveListId,
            operation = OperationType.CREATE,
            payload = AppJson.encodeToString(CreateShoppingItemRequest.serializer(), request),
            transform = { current ->
                current.updateList(effectiveListId) { list ->
                    list.copy(
                        items = list.items + item,
                        activity = list.activity.prependActivity(
                            listId = listId,
                            user = user,
                            action = "added_item",
                            itemName = name,
                            now = now,
                        ),
                    )
                }
            },
        )
        id
    }

    suspend fun updateItem(
        groupId: Long,
        listId: Long,
        itemId: Long,
        name: String,
        quantity: String?,
        category: String?,
    ): Result<Unit> = runCatching {
        val request = CreateShoppingItemRequest(name, quantity, category)
        val entry = requireSnapshot(groupId).listEntry(listId)
            ?: error("Shopping list $listId is not cached")
        val effectiveListId = entry.list.id
        val effectiveItemId = entry.item(itemId)?.id
            ?: localStore.currentId(EntityType.SHOPPING_ITEM, itemId)
        enqueue(
            groupId = groupId,
            entityType = EntityType.SHOPPING_ITEM,
            entityId = effectiveItemId,
            parentId = effectiveListId,
            operation = OperationType.UPDATE,
            payload = AppJson.encodeToString(CreateShoppingItemRequest.serializer(), request),
            transform = { current ->
                current.updateList(effectiveListId) { list ->
                    list.copy(
                        items = list.items.map { item ->
                            if (item.id != effectiveItemId) {
                                item
                            } else {
                                item.copy(
                                    name = name,
                                    quantity = quantity,
                                    category = category,
                                    updatedAt = Instant.now().toString(),
                                )
                            }
                        },
                    )
                }
            },
        )
    }

    suspend fun deleteItem(groupId: Long, listId: Long, itemId: Long): Result<Unit> =
        runCatching {
            val user = session.requireUser()
            val now = Instant.now().toString()
            val entry = requireSnapshot(groupId).listEntry(listId)
                ?: error("Shopping list $listId is not cached")
            val effectiveListId = entry.list.id
            val targetItem = entry.item(itemId)
            val effectiveItemId = targetItem?.id
                ?: localStore.currentId(EntityType.SHOPPING_ITEM, itemId)
            val itemName = targetItem?.name.orEmpty()
            enqueue(
                groupId = groupId,
                entityType = EntityType.SHOPPING_ITEM,
                entityId = effectiveItemId,
                parentId = effectiveListId,
                operation = OperationType.DELETE,
                transform = { current ->
                    current.updateList(effectiveListId) { list ->
                        list.copy(
                            items = list.items.filterNot { it.id == effectiveItemId },
                            activity = list.activity.prependActivity(
                                listId = effectiveListId,
                                user = user,
                                action = "deleted_item",
                                itemName = itemName,
                                now = now,
                            ),
                        )
                    }
                },
            )
        }

    suspend fun toggleItem(groupId: Long, listId: Long, itemId: Long): Result<Unit> =
        runCatching {
            val user = session.requireUser()
            val now = Instant.now().toString()
            val snapshot = requireSnapshot(groupId)
            val entry = snapshot.listEntry(listId)
                ?: error("Shopping list $listId is not cached")
            val effectiveListId = entry.list.id
            val item = entry.item(itemId)
                ?: error("Item $itemId is not cached")
            val effectiveItemId = item.id
            val desired = !item.isCompleted
            val action = if (desired) "completed_item" else "uncompleted_item"

            enqueue(
                groupId = groupId,
                entityType = EntityType.SHOPPING_ITEM,
                entityId = effectiveItemId,
                parentId = effectiveListId,
                operation = OperationType.TOGGLE_ITEM,
                payload = AppJson.encodeToString(
                    ToggleItemRequest.serializer(),
                    ToggleItemRequest(desired),
                ),
                transform = { current ->
                    current.updateList(effectiveListId) { list ->
                        list.copy(
                            items = list.items.map { existing ->
                                if (existing.id != effectiveItemId) {
                                    existing
                                } else {
                                    existing.copy(
                                        isCompleted = desired,
                                        completedBy = if (desired) user.id else null,
                                        completedByUsername = if (desired) user.username else null,
                                        completedAt = if (desired) now else null,
                                        updatedAt = now,
                                    )
                                }
                            },
                            activity = list.activity.prependActivity(
                                listId = effectiveListId,
                                user = user,
                                action = action,
                                itemName = item.name,
                                now = now,
                            ),
                        )
                    }
                },
            )
        }

    private suspend fun enqueue(
        groupId: Long,
        entityType: EntityType,
        entityId: Long,
        parentId: Long? = null,
        operation: OperationType,
        payload: String = "{}",
        transform: (GroupSnapshotDto) -> GroupSnapshotDto,
    ) {
        localStore.mutateGroup(
            groupId = groupId,
            operation = PendingOperation(
                groupId = groupId,
                entityType = entityType,
                entityId = entityId,
                parentId = parentId,
                operation = operation,
                payload = payload,
            ),
            transform = transform,
        )
        syncManager.requestSync()
    }

    private fun summary(
        snapshot: ShoppingListSnapshotDto,
        group: GroupSnapshotDto,
    ): ShoppingListSummaryDto {
        val creatorUsername = group.members
            .firstOrNull { it.id == snapshot.list.createdBy }
            ?.username
            .orEmpty()
        return ShoppingListSummaryDto(
            id = snapshot.list.id,
            name = snapshot.list.name,
            groupId = snapshot.list.groupId,
            createdBy = snapshot.list.createdBy,
            creatorUsername = creatorUsername,
            totalItems = snapshot.items.size.toLong(),
            completedItems = snapshot.items.count { it.isCompleted }.toLong(),
        )
    }

    private fun GroupSnapshotDto.updateList(
        listId: Long,
        transform: (ShoppingListSnapshotDto) -> ShoppingListSnapshotDto,
    ): GroupSnapshotDto = copy(
        shoppingLists = shoppingLists.map { snapshot ->
            if (snapshot.list.id == listId) transform(snapshot) else snapshot
        },
    )

    /**
     * Looks up by the id the UI holds first. Only after a pull replaced the temp
     * id does it fall back to the temp -> server mapping.
     */
    private fun GroupSnapshotDto.listEntry(listId: Long): ShoppingListSnapshotDto? =
        shoppingLists.firstOrNull { it.list.id == listId }
            ?: localStore.currentId(EntityType.SHOPPING_LIST, listId)
                .takeIf { it != listId }
                ?.let { mapped -> shoppingLists.firstOrNull { it.list.id == mapped } }

    private fun ShoppingListSnapshotDto.item(itemId: Long): ShoppingListItemDto? =
        items.firstOrNull { it.id == itemId }
            ?: localStore.currentId(EntityType.SHOPPING_ITEM, itemId)
                .takeIf { it != itemId }
                ?.let { mapped -> items.firstOrNull { it.id == mapped } }

    private fun List<ShoppingListActivityDto>.prependActivity(
        listId: Long,
        user: com.splitify.app.data.dto.UserDto,
        action: String,
        itemName: String,
        now: String,
    ): List<ShoppingListActivityDto> = (
        listOf(
            ShoppingListActivityDto(
                id = TempIds.next(),
                shoppingListId = listId,
                userId = user.id,
                username = user.username,
                action = action,
                itemName = itemName,
                createdAt = now,
            ),
        ) + this
        ).take(ACTIVITY_LIMIT)

    private suspend fun requireSnapshot(groupId: Long): GroupSnapshotDto =
        localStore.groupSnapshot(groupId)
            ?: error("Group $groupId is not cached yet")

    private companion object {
        const val ACTIVITY_LIMIT = 10
    }
}
