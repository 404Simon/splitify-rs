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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import androidx.lifecycle.viewmodel.compose.viewModel
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.debts.DebtRepository
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.SharedDebtDto
import com.splitify.app.data.dto.UserShareDto
import com.splitify.app.data.network.ApiException
import com.splitify.app.data.network.userMessage
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

private val AmountPattern = Regex("""^\d+(\.\d{1,2})?$""")

data class DebtsUiState(
    val debts: List<SharedDebtDto> = emptyList(),
    val shares: Map<Long, List<UserShareDto>> = emptyMap(),
    val isLoading: Boolean = true,
    val error: String? = null,
    val showFormDialog: Boolean = false,
    val editingDebtId: Long? = null,
    val isSaving: Boolean = false,
    val formError: String? = null,
    val deleteTarget: SharedDebtDto? = null,
    val isDeleting: Boolean = false,
)

class DebtsViewModel(
    private val groupId: Long,
    private val debtRepository: DebtRepository,
    private val authRepository: AuthRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(DebtsUiState())
    val uiState: StateFlow<DebtsUiState> = _uiState.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            debtRepository.listDebts(groupId)
                .onSuccess { debts ->
                    val shares = coroutineScope {
                        debts.map { debt ->
                            async {
                                debt.id to debtRepository.getDebtShares(debt.id).getOrDefault(emptyList())
                            }
                        }.awaitAll().toMap()
                    }
                    _uiState.update {
                        it.copy(debts = debts, shares = shares, isLoading = false)
                    }
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update { it.copy(isLoading = false, error = error.userMessage()) }
                }
        }
    }

    fun openCreate() {
        _uiState.update {
            it.copy(showFormDialog = true, editingDebtId = null, formError = null)
        }
    }

    fun openEdit(debt: SharedDebtDto) {
        _uiState.update {
            it.copy(showFormDialog = true, editingDebtId = debt.id, formError = null)
        }
    }

    fun dismissForm() {
        if (_uiState.value.isSaving) return
        _uiState.update { it.copy(showFormDialog = false, editingDebtId = null, formError = null) }
    }

    fun save(name: String, amount: String, memberIds: List<Long>) {
        val trimmedName = name.trim()
        val trimmedAmount = amount.trim()
        val validationError = when {
            trimmedName.isEmpty() -> "Enter a name."
            !AmountPattern.matches(trimmedAmount) -> "Enter a valid amount with up to 2 decimals."
            memberIds.isEmpty() -> "Select at least one member."
            else -> null
        }
        if (validationError != null) {
            _uiState.update { it.copy(formError = validationError) }
            return
        }
        if (_uiState.value.isSaving) return
        _uiState.update { it.copy(isSaving = true, formError = null) }

        viewModelScope.launch {
            val editingId = _uiState.value.editingDebtId
            val result: Result<Unit> = if (editingId != null) {
                debtRepository.updateDebt(editingId, trimmedName, trimmedAmount, memberIds)
            } else {
                debtRepository.createDebt(groupId, trimmedName, trimmedAmount, memberIds).map { }
            }
            result
                .onSuccess {
                    _uiState.update {
                        it.copy(
                            isSaving = false,
                            showFormDialog = false,
                            editingDebtId = null,
                            formError = null,
                        )
                    }
                    load()
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update {
                        it.copy(isSaving = false, formError = error.userMessage())
                    }
                }
        }
    }

    fun requestDelete(debt: SharedDebtDto) {
        _uiState.update { it.copy(deleteTarget = debt) }
    }

    fun dismissDelete() {
        if (_uiState.value.isDeleting) return
        _uiState.update { it.copy(deleteTarget = null) }
    }

    fun confirmDelete() {
        val target = _uiState.value.deleteTarget ?: return
        if (_uiState.value.isDeleting) return
        _uiState.update { it.copy(isDeleting = true) }
        viewModelScope.launch {
            debtRepository.deleteDebt(target.id)
                .onSuccess {
                    _uiState.update { it.copy(isDeleting = false, deleteTarget = null) }
                    load()
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update {
                        it.copy(
                            isDeleting = false,
                            deleteTarget = null,
                            error = error.userMessage(),
                        )
                    }
                }
        }
    }

    private suspend fun handleUnauthorized(error: Throwable) {
        if (error is ApiException.Http && error.isUnauthorized) {
            authRepository.clearSession()
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            debtRepository: DebtRepository,
            authRepository: AuthRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { DebtsViewModel(groupId, debtRepository, authRepository) }
        }
    }
}

@Composable
fun DebtsTab(
    groupId: Long,
    members: List<GroupMemberDto>,
    debtRepository: DebtRepository,
    authRepository: AuthRepository,
) {
    val viewModel: DebtsViewModel = viewModel(
        key = "debts-$groupId",
        factory = DebtsViewModel.factory(groupId, debtRepository, authRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.load() }

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.debts.isEmpty() -> LoadingScreen(message = "Loading debts…")

            state.error != null && state.debts.isEmpty() -> TabError(
                message = state.error.orEmpty(),
                onRetry = { viewModel.load() },
            )

            state.debts.isEmpty() -> EmptyDebts(onCreate = { viewModel.openCreate() })

            else -> Column(modifier = Modifier.fillMaxSize()) {
                state.error?.let { message ->
                    ErrorBanner(
                        message = message,
                        onRetry = { viewModel.load() },
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
                    items(state.debts, key = { it.id }) { debt ->
                        DebtCard(
                            debt = debt,
                            shares = state.shares[debt.id].orEmpty(),
                            onEdit = { viewModel.openEdit(debt) },
                            onDelete = { viewModel.requestDelete(debt) },
                        )
                    }
                }
            }
        }

        if (!state.isLoading || state.debts.isNotEmpty()) {
            ExtendedFloatingActionButton(
                onClick = { viewModel.openCreate() },
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(16.dp),
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("New debt") },
            )
        }
    }

    if (state.showFormDialog) {
        val editingDebt = state.debts.firstOrNull { it.id == state.editingDebtId }
        val initialShareIds = editingDebt
            ?.let { debt -> state.shares[debt.id]?.map { it.userId } }
            ?.takeIf { it.isNotEmpty() }
            ?: members.map { it.id }
        DebtFormDialog(
            isEditing = editingDebt != null,
            initialName = editingDebt?.name.orEmpty(),
            initialAmount = editingDebt?.amount.orEmpty(),
            initialMemberIds = initialShareIds,
            members = members,
            isSaving = state.isSaving,
            error = state.formError,
            onDismiss = { viewModel.dismissForm() },
            onSave = { name, amount, memberIds -> viewModel.save(name, amount, memberIds) },
        )
    }

    state.deleteTarget?.let { debt ->
        AlertDialog(
            onDismissRequest = { viewModel.dismissDelete() },
            title = { Text("Delete debt") },
            text = { Text("Delete \"${debt.name}\"? This cannot be undone.") },
            confirmButton = {
                TextButton(
                    onClick = { viewModel.confirmDelete() },
                    enabled = !state.isDeleting,
                ) {
                    if (state.isDeleting) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(18.dp),
                            strokeWidth = 2.dp,
                        )
                    } else {
                        Text("Delete", color = MaterialTheme.colorScheme.error)
                    }
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.dismissDelete() }, enabled = !state.isDeleting) {
                    Text("Cancel")
                }
            },
        )
    }
}

@Composable
private fun DebtCard(
    debt: SharedDebtDto,
    shares: List<UserShareDto>,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(start = 16.dp, end = 8.dp, top = 12.dp, bottom = 12.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = debt.name,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = "Created by ${debt.creatorUsername}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Text(
                    text = debt.amount,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.primary,
                )
                if (debt.isCreator) {
                    IconButton(onClick = onEdit) {
                        Icon(Icons.Default.Edit, contentDescription = "Edit debt")
                    }
                    IconButton(onClick = onDelete) {
                        Icon(
                            Icons.Default.Delete,
                            contentDescription = "Delete debt",
                            tint = MaterialTheme.colorScheme.error,
                        )
                    }
                }
            }

            if (shares.isNotEmpty()) {
                Text(
                    text = "Split: " + shares.joinToString(" · ") {
                        "${it.username} ${it.shareAmount}"
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun EmptyDebts(onCreate: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Default.Warning,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(40.dp),
        )
        Spacer(Modifier.width(12.dp))
        Text(text = "No shared debts yet", style = MaterialTheme.typography.titleMedium)
        Text(
            text = "Add a shared expense to split it among group members.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = 4.dp),
        )
        Button(onClick = onCreate, modifier = Modifier.padding(top = 16.dp)) {
            Text("Add debt")
        }
    }
}

@Composable
private fun TabError(
    message: String,
    onRetry: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        verticalArrangement = Arrangement.Center,
    ) {
        ErrorBanner(message = message, onRetry = onRetry)
    }
}

@Composable
private fun DebtFormDialog(
    isEditing: Boolean,
    initialName: String,
    initialAmount: String,
    initialMemberIds: List<Long>,
    members: List<GroupMemberDto>,
    isSaving: Boolean,
    error: String?,
    onDismiss: () -> Unit,
    onSave: (String, String, List<Long>) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf(initialName) }
    var amount by rememberSaveable { mutableStateOf(initialAmount) }
    val selectedMemberIds = remember { mutableStateListOf<Long>().apply { addAll(initialMemberIds) } }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(if (isEditing) "Edit debt" else "New debt") },
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
                    enabled = !isSaving,
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = amount,
                    onValueChange = { amount = it },
                    label = { Text("Amount") },
                    singleLine = true,
                    enabled = !isSaving,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    modifier = Modifier.fillMaxWidth(),
                )
                Text(
                    text = "Split between",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                members.forEach { member ->
                    val checked = selectedMemberIds.contains(member.id)
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable(enabled = !isSaving) {
                                if (checked) {
                                    selectedMemberIds.remove(member.id)
                                } else {
                                    selectedMemberIds.add(member.id)
                                }
                            },
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(
                            checked = checked,
                            onCheckedChange = { value ->
                                if (value) {
                                    if (!selectedMemberIds.contains(member.id)) {
                                        selectedMemberIds.add(member.id)
                                    }
                                } else {
                                    selectedMemberIds.remove(member.id)
                                }
                            },
                            enabled = !isSaving,
                        )
                        Text(text = member.username)
                    }
                }
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
                onClick = { onSave(name, amount, selectedMemberIds.toList()) },
                enabled = !isSaving &&
                    name.isNotBlank() &&
                    amount.isNotBlank() &&
                    selectedMemberIds.isNotEmpty(),
            ) {
                if (isSaving) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        strokeWidth = 2.dp,
                    )
                } else {
                    Text(if (isEditing) "Save" else "Create")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isSaving) {
                Text("Cancel")
            }
        },
    )
}
