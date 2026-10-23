package com.splitify.app.ui.groups.tabs

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.ShoppingCart
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.dto.ShoppingListActivityDto
import com.splitify.app.data.dto.ShoppingListDto
import com.splitify.app.data.dto.ShoppingListItemDto
import com.splitify.app.data.dto.ShoppingListSummaryDto
import com.splitify.app.data.shoppinglists.ShoppingListRepository
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import kotlinx.coroutines.Job
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * Snapshot of the tab. Everything rendered here comes from [ShoppingListRepository]
 * flows (the local cache), so the UI never waits for the network.
 */
data class ShoppingListsUiState(
    val lists: List<ShoppingListSummaryDto> = emptyList(),
    val selectedListId: Long? = null,
    val selectedList: ShoppingListDto? = null,
    val items: List<ShoppingListItemDto> = emptyList(),
    val activity: List<ShoppingListActivityDto> = emptyList(),
    val isLoading: Boolean = true,
    val error: String? = null,
    val isLoadingDetail: Boolean = false,
    val detailError: String? = null,
    val showCreateDialog: Boolean = false,
    val createError: String? = null,
    val renameTarget: ShoppingListSummaryDto? = null,
    val renameError: String? = null,
    val deleteTarget: ShoppingListSummaryDto? = null,
    val showItemDialog: Boolean = false,
    val editingItem: ShoppingListItemDto? = null,
    val itemError: String? = null,
    val deleteItemTarget: ShoppingListItemDto? = null,
)

class ShoppingListsViewModel(
    private val groupId: Long,
    private val shoppingListRepository: ShoppingListRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(ShoppingListsUiState())
    val uiState: StateFlow<ShoppingListsUiState> = _uiState.asStateFlow()

    private var detailJob: Job? = null

    /**
     * Taps waiting for the local snapshot write to land. Guards against a fast
     * double tap reading the same pre-toggle state twice.
     */
    private val inFlightToggles = mutableSetOf<Long>()

    init {
        viewModelScope.launch {
            shoppingListRepository.observeLists(groupId).collect { lists ->
                _uiState.update { it.copy(lists = lists, isLoading = false) }
            }
        }
    }

    fun selectList(listId: Long) {
        if (_uiState.value.selectedListId == listId) return
        _uiState.update {
            it.copy(
                selectedListId = listId,
                selectedList = null,
                items = emptyList(),
                activity = emptyList(),
                isLoadingDetail = true,
                detailError = null,
            )
        }
        observeDetail(listId)
    }

    fun closeDetail() {
        detailJob?.cancel()
        detailJob = null
        _uiState.update {
            it.copy(
                selectedListId = null,
                selectedList = null,
                items = emptyList(),
                activity = emptyList(),
                isLoadingDetail = false,
                detailError = null,
                showItemDialog = false,
                editingItem = null,
                itemError = null,
                deleteItemTarget = null,
            )
        }
    }

    fun dismissError() {
        _uiState.update { it.copy(error = null) }
    }

    fun dismissDetailError() {
        _uiState.update { it.copy(detailError = null) }
    }

    fun openCreate() {
        _uiState.update { it.copy(showCreateDialog = true, createError = null) }
    }

    fun dismissCreate() {
        _uiState.update { it.copy(showCreateDialog = false, createError = null) }
    }

    fun createList(name: String) {
        val trimmed = name.trim()
        if (trimmed.isEmpty()) {
            _uiState.update { it.copy(createError = "Enter a list name.") }
            return
        }
        viewModelScope.launch {
            shoppingListRepository.createList(groupId, trimmed)
                .onSuccess { listId ->
                    _uiState.update { it.copy(showCreateDialog = false, createError = null) }
                    selectList(listId)
                }
                .onFailure {
                    _uiState.update { it.copy(createError = "Couldn't create the list.") }
                }
        }
    }

    fun requestRename(list: ShoppingListSummaryDto) {
        _uiState.update { it.copy(renameTarget = list, renameError = null) }
    }

    fun dismissRename() {
        _uiState.update { it.copy(renameTarget = null, renameError = null) }
    }

    fun renameList(name: String) {
        val target = _uiState.value.renameTarget ?: return
        val trimmed = name.trim()
        if (trimmed.isEmpty()) {
            _uiState.update { it.copy(renameError = "Enter a list name.") }
            return
        }
        _uiState.update { it.copy(renameTarget = null, renameError = null) }
        viewModelScope.launch {
            shoppingListRepository.updateList(groupId, target.id, trimmed)
                .onFailure {
                    _uiState.update { it.copy(error = "Couldn't rename the list.") }
                }
        }
    }

    fun requestDelete(list: ShoppingListSummaryDto) {
        _uiState.update { it.copy(deleteTarget = list) }
    }

    fun dismissDelete() {
        _uiState.update { it.copy(deleteTarget = null) }
    }

    fun confirmDelete() {
        val target = _uiState.value.deleteTarget ?: return
        _uiState.update { it.copy(deleteTarget = null) }
        viewModelScope.launch {
            shoppingListRepository.deleteList(groupId, target.id)
                .onSuccess {
                    val state = _uiState.value
                    if (state.selectedListId == target.id || state.selectedList?.id == target.id) {
                        closeDetail()
                    }
                }
                .onFailure {
                    _uiState.update { it.copy(error = "Couldn't delete the list.") }
                }
        }
    }

    fun openAddItem() {
        _uiState.update { it.copy(showItemDialog = true, editingItem = null, itemError = null) }
    }

    fun openEditItem(item: ShoppingListItemDto) {
        _uiState.update { it.copy(showItemDialog = true, editingItem = item, itemError = null) }
    }

    fun dismissItemForm() {
        _uiState.update { it.copy(showItemDialog = false, editingItem = null, itemError = null) }
    }

    fun saveItem(name: String, quantity: String, category: String) {
        val listId = _uiState.value.selectedListId ?: return
        val trimmedName = name.trim()
        if (trimmedName.isEmpty()) {
            _uiState.update { it.copy(itemError = "Enter an item name.") }
            return
        }
        val editingItem = _uiState.value.editingItem
        val trimmedQuantity = quantity.trim().ifEmpty { null }
        val trimmedCategory = category.trim().ifEmpty { null }

        _uiState.update { it.copy(showItemDialog = false, editingItem = null, itemError = null) }
        viewModelScope.launch {
            val result = if (editingItem != null) {
                shoppingListRepository.updateItem(
                    groupId = groupId,
                    listId = listId,
                    itemId = editingItem.id,
                    name = trimmedName,
                    quantity = trimmedQuantity,
                    category = trimmedCategory,
                )
            } else {
                shoppingListRepository.addItem(
                    groupId = groupId,
                    listId = listId,
                    name = trimmedName,
                    quantity = trimmedQuantity,
                    category = trimmedCategory,
                ).map { }
            }
            if (result.isFailure) {
                updateDetail(listId) { it.copy(detailError = "Couldn't save the item.") }
            }
        }
    }

    fun requestDeleteItem(item: ShoppingListItemDto) {
        _uiState.update { it.copy(deleteItemTarget = item) }
    }

    fun dismissDeleteItem() {
        _uiState.update { it.copy(deleteItemTarget = null) }
    }

    fun confirmDeleteItem() {
        val listId = _uiState.value.selectedListId ?: return
        val target = _uiState.value.deleteItemTarget ?: return
        _uiState.update { it.copy(deleteItemTarget = null) }
        viewModelScope.launch {
            shoppingListRepository.deleteItem(groupId, listId, target.id)
                .onFailure {
                    updateDetail(listId) { it.copy(detailError = "Couldn't delete the item.") }
                }
        }
    }

    fun toggleItem(item: ShoppingListItemDto) {
        val listId = _uiState.value.selectedListId ?: return
        if (!inFlightToggles.add(item.id)) return
        viewModelScope.launch {
            try {
                shoppingListRepository.toggleItem(groupId, listId, item.id)
                    .onFailure {
                        updateDetail(listId) { it.copy(detailError = "Couldn't update the item.") }
                    }
            } finally {
                inFlightToggles.remove(item.id)
            }
        }
    }

    private fun observeDetail(listId: Long) {
        detailJob?.cancel()
        detailJob = viewModelScope.launch {
            coroutineScope {
                launch {
                    shoppingListRepository.observeList(groupId, listId).collect { list ->
                        updateDetail(listId) {
                            it.copy(selectedList = list, isLoadingDetail = false)
                        }
                    }
                }
                launch {
                    shoppingListRepository.observeItems(groupId, listId).collect { items ->
                        updateDetail(listId) {
                            it.copy(items = items, isLoadingDetail = false)
                        }
                    }
                }
                launch {
                    shoppingListRepository.observeActivity(groupId, listId).collect { activity ->
                        updateDetail(listId) { it.copy(activity = activity) }
                    }
                }
            }
        }
    }

    private fun updateDetail(
        listId: Long,
        transform: (ShoppingListsUiState) -> ShoppingListsUiState,
    ) {
        _uiState.update { state ->
            if (state.selectedListId == listId) transform(state) else state
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            shoppingListRepository: ShoppingListRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { ShoppingListsViewModel(groupId, shoppingListRepository) }
        }
    }
}

@Composable
fun ShoppingListsTab(
    groupId: Long,
    currentUserId: Long?,
    shoppingListRepository: ShoppingListRepository,
) {
    val viewModel: ShoppingListsViewModel = viewModel(
        key = "shopping-lists-$groupId",
        factory = ShoppingListsViewModel.factory(groupId, shoppingListRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    if (state.selectedListId != null) {
        ShoppingListDetail(
            state = state,
            currentUserId = currentUserId,
            viewModel = viewModel,
        )
    } else {
        ShoppingListsOverview(
            state = state,
            currentUserId = currentUserId,
            viewModel = viewModel,
        )
    }

    if (state.showCreateDialog) {
        NameDialog(
            title = "New shopping list",
            label = "List name",
            initialName = "",
            error = state.createError,
            confirmLabel = "Create",
            onDismiss = { viewModel.dismissCreate() },
            onConfirm = { viewModel.createList(it) },
        )
    }

    state.renameTarget?.let { target ->
        NameDialog(
            title = "Rename list",
            label = "List name",
            initialName = target.name,
            error = state.renameError,
            confirmLabel = "Save",
            onDismiss = { viewModel.dismissRename() },
            onConfirm = { viewModel.renameList(it) },
        )
    }

    state.deleteTarget?.let { target ->
        AlertDialog(
            onDismissRequest = { viewModel.dismissDelete() },
            title = { Text("Delete shopping list") },
            text = { Text("Delete \"${target.name}\"? All of its items will be removed.") },
            confirmButton = {
                TextButton(onClick = { viewModel.confirmDelete() }) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.dismissDelete() }) {
                    Text("Cancel")
                }
            },
        )
    }

    if (state.showItemDialog) {
        ShoppingItemDialog(
            editingItem = state.editingItem,
            error = state.itemError,
            onDismiss = { viewModel.dismissItemForm() },
            onSave = { name, quantity, category ->
                viewModel.saveItem(name, quantity, category)
            },
        )
    }

    state.deleteItemTarget?.let { item ->
        AlertDialog(
            onDismissRequest = { viewModel.dismissDeleteItem() },
            title = { Text("Delete item") },
            text = { Text("Delete \"${item.name}\" from this list?") },
            confirmButton = {
                TextButton(onClick = { viewModel.confirmDeleteItem() }) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.dismissDeleteItem() }) {
                    Text("Cancel")
                }
            },
        )
    }
}

@Composable
private fun ShoppingListsOverview(
    state: ShoppingListsUiState,
    currentUserId: Long?,
    viewModel: ShoppingListsViewModel,
) {
    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.lists.isEmpty() ->
                LoadingScreen(message = "Loading shopping lists…")

            state.lists.isEmpty() -> EmptyShoppingLists(
                onCreate = { viewModel.openCreate() },
            )

            else -> Column(modifier = Modifier.fillMaxSize()) {
                state.error?.let { message ->
                    ErrorBanner(
                        message = message,
                        onDismiss = { viewModel.dismissError() },
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                    )
                }
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(
                        start = 16.dp,
                        end = 16.dp,
                        top = 8.dp,
                        bottom = 96.dp,
                    ),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    items(state.lists, key = { it.id }) { list ->
                        ShoppingListCard(
                            list = list,
                            canDelete = currentUserId != null && currentUserId == list.createdBy,
                            onOpen = { viewModel.selectList(list.id) },
                            onRename = { viewModel.requestRename(list) },
                            onDelete = { viewModel.requestDelete(list) },
                        )
                    }
                }
            }
        }

        if (!state.isLoading || state.lists.isNotEmpty()) {
            ExtendedFloatingActionButton(
                onClick = { viewModel.openCreate() },
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(16.dp),
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("New list") },
            )
        }
    }
}

@Composable
private fun ShoppingListCard(
    list: ShoppingListSummaryDto,
    canDelete: Boolean,
    onOpen: () -> Unit,
    onRename: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onOpen),
    ) {
        Column(
            modifier = Modifier.padding(start = 16.dp, end = 8.dp, top = 12.dp, bottom = 12.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = list.name,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = "Created by ${list.creatorUsername}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                IconButton(onClick = onRename) {
                    Icon(Icons.Default.Edit, contentDescription = "Rename list")
                }
                if (canDelete) {
                    IconButton(onClick = onDelete) {
                        Icon(
                            Icons.Default.Delete,
                            contentDescription = "Delete list",
                            tint = MaterialTheme.colorScheme.error,
                        )
                    }
                }
            }

            val progress = if (list.totalItems > 0) {
                list.completedItems.toFloat() / list.totalItems.toFloat()
            } else {
                0f
            }
            if (list.totalItems > 0) {
                LinearProgressIndicator(
                    progress = { progress },
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            Text(
                text = if (list.totalItems > 0) {
                    "${list.completedItems} of ${list.totalItems} items done"
                } else {
                    "No items yet"
                },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun ShoppingListDetail(
    state: ShoppingListsUiState,
    currentUserId: Long?,
    viewModel: ShoppingListsViewModel,
) {
    val list = state.selectedList
    val canDelete = list != null && currentUserId != null && list.createdBy == currentUserId
    val summary = list?.let { summaryOf(it, state) }

    Column(modifier = Modifier.fillMaxSize()) {
        Surface(color = MaterialTheme.colorScheme.surfaceVariant) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 4.dp, vertical = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(onClick = { viewModel.closeDetail() }) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                        contentDescription = "Back to shopping lists",
                    )
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = list?.name ?: "Shopping list",
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    val done = state.items.count { it.isCompleted }
                    Text(
                        text = "$done of ${state.items.size} items done",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (summary != null) {
                    IconButton(onClick = { viewModel.requestRename(summary) }) {
                        Icon(Icons.Default.Edit, contentDescription = "Rename list")
                    }
                    if (canDelete) {
                        IconButton(onClick = { viewModel.requestDelete(summary) }) {
                            Icon(
                                Icons.Default.Delete,
                                contentDescription = "Delete list",
                                tint = MaterialTheme.colorScheme.error,
                            )
                        }
                    }
                }
            }
        }

        state.detailError?.let { message ->
            ErrorBanner(
                message = message,
                onDismiss = { viewModel.dismissDetailError() },
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
            )
        }

        Box(modifier = Modifier.weight(1f)) {
            when {
                state.isLoadingDetail && state.items.isEmpty() ->
                    LoadingScreen(message = "Loading items…")

                list == null && state.items.isEmpty() ->
                    MissingShoppingList(onBack = { viewModel.closeDetail() })

                else -> LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(
                        start = 16.dp,
                        end = 16.dp,
                        top = 8.dp,
                        bottom = 96.dp,
                    ),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    if (state.items.isEmpty()) {
                        item(key = "empty-items") {
                            Text(
                                text = "No items yet. Add the first thing to buy.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.padding(vertical = 16.dp),
                            )
                        }
                    } else {
                        items(state.items, key = { "item-${it.id}" }) { item ->
                            ShoppingItemRow(
                                item = item,
                                onToggle = { viewModel.toggleItem(item) },
                                onEdit = { viewModel.openEditItem(item) },
                                onDelete = { viewModel.requestDeleteItem(item) },
                            )
                        }
                    }

                    if (state.activity.isNotEmpty()) {
                        item(key = "activity-title") {
                            Text(
                                text = "Recent activity",
                                style = MaterialTheme.typography.titleSmall,
                                modifier = Modifier.padding(top = 12.dp),
                            )
                        }
                        items(state.activity, key = { "activity-${it.id}" }) { entry ->
                            ActivityRow(entry = entry)
                        }
                    }
                }
            }

            if (list != null) {
                ExtendedFloatingActionButton(
                    onClick = { viewModel.openAddItem() },
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(16.dp),
                    icon = { Icon(Icons.Default.Add, contentDescription = null) },
                    text = { Text("Add item") },
                )
            }
        }
    }
}

@Composable
private fun ShoppingItemRow(
    item: ShoppingListItemDto,
    onToggle: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.padding(start = 4.dp, end = 4.dp, top = 4.dp, bottom = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Checkbox(
                checked = item.isCompleted,
                onCheckedChange = { onToggle() },
            )
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(vertical = 6.dp),
            ) {
                Text(
                    text = item.name,
                    style = MaterialTheme.typography.bodyLarge,
                    textDecoration = if (item.isCompleted) {
                        TextDecoration.LineThrough
                    } else {
                        TextDecoration.None
                    },
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                val meta = listOfNotNull(
                    item.quantity?.takeIf { it.isNotBlank() },
                    item.category?.takeIf { it.isNotBlank() },
                ).joinToString(" · ")
                if (meta.isNotEmpty()) {
                    Text(
                        text = meta,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (item.isCompleted) {
                    item.completedByUsername?.let { username ->
                        Text(
                            text = "Completed by $username",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
            IconButton(onClick = onEdit) {
                Icon(Icons.Default.Edit, contentDescription = "Edit item")
            }
            IconButton(onClick = onDelete) {
                Icon(
                    Icons.Default.Delete,
                    contentDescription = "Delete item",
                    tint = MaterialTheme.colorScheme.error,
                )
            }
        }
    }
}

@Composable
private fun ActivityRow(entry: ShoppingListActivityDto) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = activityDescription(entry),
                style = MaterialTheme.typography.bodySmall,
            )
            Text(
                text = entry.createdAt.take(16).replace('T', ' '),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun EmptyShoppingLists(onCreate: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Default.ShoppingCart,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(40.dp),
        )
        Spacer(Modifier.height(12.dp))
        Text(text = "No shopping lists yet", style = MaterialTheme.typography.titleMedium)
        Text(
            text = "Create a shared list for groceries, supplies, or anything else.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = 4.dp),
        )
        Button(onClick = onCreate, modifier = Modifier.padding(top = 16.dp)) {
            Text("Create list")
        }
    }
}

@Composable
private fun MissingShoppingList(onBack: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(
            text = "This shopping list is no longer available.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
        TextButton(onClick = onBack, modifier = Modifier.padding(top = 8.dp)) {
            Text("Back to lists")
        }
    }
}

@Composable
private fun NameDialog(
    title: String,
    label: String,
    initialName: String,
    error: String?,
    confirmLabel: String,
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit,
) {
    var name by remember { mutableStateOf(initialName) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(title) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(label) },
                    singleLine = true,
                    isError = error != null,
                    modifier = Modifier.fillMaxWidth(),
                )
                error?.let {
                    Text(
                        text = it,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = { onConfirm(name) },
                enabled = name.isNotBlank(),
            ) {
                Text(confirmLabel)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        },
    )
}

@Composable
private fun ShoppingItemDialog(
    editingItem: ShoppingListItemDto?,
    error: String?,
    onDismiss: () -> Unit,
    onSave: (String, String, String) -> Unit,
) {
    var name by remember(editingItem?.id) { mutableStateOf(editingItem?.name.orEmpty()) }
    var quantity by remember(editingItem?.id) { mutableStateOf(editingItem?.quantity.orEmpty()) }
    var category by remember(editingItem?.id) { mutableStateOf(editingItem?.category.orEmpty()) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(if (editingItem == null) "Add item" else "Edit item") },
        text = {
            Column(
                modifier = Modifier
                    .heightIn(max = 420.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Name") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = quantity,
                    onValueChange = { quantity = it },
                    label = { Text("Quantity (optional)") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Text),
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = category,
                    onValueChange = { category = it },
                    label = { Text("Category (optional)") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                error?.let {
                    Text(
                        text = it,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = { onSave(name, quantity, category) },
                enabled = name.isNotBlank(),
            ) {
                Text(if (editingItem == null) "Add" else "Save")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel")
            }
        },
    )
}

private fun summaryOf(
    list: ShoppingListDto,
    state: ShoppingListsUiState,
): ShoppingListSummaryDto {
    val existing = state.lists.firstOrNull { it.id == list.id }
    return existing ?: ShoppingListSummaryDto(
        id = list.id,
        name = list.name,
        groupId = list.groupId,
        createdBy = list.createdBy,
        creatorUsername = "",
        totalItems = state.items.size.toLong(),
        completedItems = state.items.count { it.isCompleted }.toLong(),
    )
}

private fun activityDescription(entry: ShoppingListActivityDto): String = when (entry.action) {
    "added_item" -> "${entry.username} added ${entry.itemName}"
    "completed_item" -> "${entry.username} completed ${entry.itemName}"
    "uncompleted_item" -> "${entry.username} reopened ${entry.itemName}"
    "deleted_item" -> "${entry.username} deleted ${entry.itemName}"
    else -> "${entry.username} ${entry.action.replace('_', ' ')} ${entry.itemName}"
}
