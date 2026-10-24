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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import androidx.lifecycle.viewmodel.compose.viewModel
import com.splitify.app.data.debts.DebtRepository
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.SharedDebtDto
import com.splitify.app.data.dto.UserShareDto
import com.splitify.app.data.network.userMessage
import com.splitify.app.ui.components.Avatar
import com.splitify.app.ui.components.AvatarStack
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.IconBubble
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.components.MoneyText
import com.splitify.app.ui.components.SplitifyCard
import com.splitify.app.ui.format.formatMoney
import com.splitify.app.ui.format.splitEvenly
import com.splitify.app.ui.theme.splitify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

private val AmountPattern = Regex("""^\d+(\.\d{1,2})?$""")

data class DebtsUiState(
    val debts: List<SharedDebtDto> = emptyList(),
    val shares: Map<Long, List<UserShareDto>> = emptyMap(),
    val isLoading: Boolean = true,
    val actionError: String? = null,
    val showFormDialog: Boolean = false,
    val editingDebtId: Long? = null,
    val isSaving: Boolean = false,
    val formError: String? = null,
    val deleteTarget: SharedDebtDto? = null,
)

class DebtsViewModel(
    private val groupId: Long,
    private val debtRepository: DebtRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(DebtsUiState())
    val uiState: StateFlow<DebtsUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            combine(
                debtRepository.observeDebts(groupId),
                debtRepository.observeShares(groupId),
            ) { debts, shares -> debts to shares }
                .collect { (debts, shares) ->
                    _uiState.update {
                        it.copy(debts = debts, shares = shares, isLoading = false)
                    }
                }
        }
    }

    fun dismissActionError() {
        _uiState.update { it.copy(actionError = null) }
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
                debtRepository.updateDebt(
                    groupId = groupId,
                    debtId = editingId,
                    name = trimmedName,
                    amount = trimmedAmount,
                    memberIds = memberIds,
                )
            } else {
                debtRepository.createDebt(
                    groupId = groupId,
                    name = trimmedName,
                    amount = trimmedAmount,
                    memberIds = memberIds,
                ).map { }
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
                }
                .onFailure { error ->
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
        _uiState.update { it.copy(deleteTarget = null) }
    }

    fun confirmDelete() {
        val target = _uiState.value.deleteTarget ?: return
        // The cached list is the source of truth, so the dialog can close before the
        // local write completes and no server round-trip is involved.
        _uiState.update { it.copy(deleteTarget = null) }
        viewModelScope.launch {
            debtRepository.deleteDebt(groupId, target.id)
                .onFailure { error ->
                    _uiState.update { it.copy(actionError = error.userMessage()) }
                }
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            debtRepository: DebtRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { DebtsViewModel(groupId, debtRepository) }
        }
    }
}

@Composable
fun DebtsTab(
    groupId: Long,
    members: List<GroupMemberDto>,
    debtRepository: DebtRepository,
) {
    val viewModel: DebtsViewModel = viewModel(
        key = "debts-$groupId",
        factory = DebtsViewModel.factory(groupId, debtRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.debts.isEmpty() -> LoadingScreen(message = "Loading debts…")

            state.debts.isEmpty() -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                EmptyState(
                    icon = Icons.AutoMirrored.Filled.List,
                    title = "No shared debts yet",
                    message = "Add an expense once and Splitify works out who owes what.",
                    actionLabel = "Add a debt",
                    onAction = { viewModel.openCreate() },
                )
            }

            else -> Column(modifier = Modifier.fillMaxSize()) {
                state.actionError?.let { message ->
                    ErrorBanner(
                        message = message,
                        onDismiss = { viewModel.dismissActionError() },
                        modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
                    )
                }
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(
                        start = 20.dp,
                        end = 20.dp,
                        top = 4.dp,
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
                shape = CircleShape,
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
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
            shape = RoundedCornerShape(28.dp),
            title = { Text("Delete debt") },
            text = { Text("Delete \"${debt.name}\"? This cannot be undone.") },
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
}

@Composable
private fun DebtCard(
    debt: SharedDebtDto,
    shares: List<UserShareDto>,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }

    SplitifyCard(contentPadding = PaddingValues(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            IconBubble(
                icon = Icons.AutoMirrored.Filled.List,
                size = 44.dp,
                containerColor = MaterialTheme.colorScheme.secondaryContainer,
                contentColor = MaterialTheme.colorScheme.onSecondaryContainer,
            )
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = debt.name,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = "Added by ${debt.creatorUsername}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Column(horizontalAlignment = Alignment.End) {
                MoneyText(
                    amount = debt.amount,
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.primary,
                )
                Text(
                    text = "total",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            if (debt.isCreator) {
                Box {
                    IconButton(onClick = { menuExpanded = true }) {
                        Icon(
                            imageVector = Icons.Default.MoreVert,
                            contentDescription = "Debt options",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    DropdownMenu(
                        expanded = menuExpanded,
                        onDismissRequest = { menuExpanded = false },
                    ) {
                        DropdownMenuItem(
                            text = { Text("Edit") },
                            leadingIcon = { Icon(Icons.Default.Edit, contentDescription = null) },
                            onClick = {
                                menuExpanded = false
                                onEdit()
                            },
                        )
                        DropdownMenuItem(
                            text = { Text("Delete") },
                            leadingIcon = {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.error,
                                )
                            },
                            onClick = {
                                menuExpanded = false
                                onDelete()
                            },
                        )
                    }
                }
            }
        }

        if (shares.isNotEmpty()) {
            Spacer(Modifier.size(12.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                AvatarStack(
                    people = shares.map { it.userId to it.username },
                    size = 26.dp,
                    max = 4,
                    ringColor = MaterialTheme.splitify.card,
                )
                Spacer(Modifier.width(10.dp))
                val each = shares.map { it.shareAmount }.distinct().singleOrNull()
                Text(
                    text = if (each != null) {
                        "${formatMoney(each)} each · split ${shares.size} ways"
                    } else {
                        "Split ${shares.size} ways"
                    },
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
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
    val eachShare = splitEvenly(amount, selectedMemberIds.size)

    AlertDialog(
        onDismissRequest = onDismiss,
        shape = RoundedCornerShape(28.dp),
        title = { Text(if (isEditing) "Edit debt" else "New debt") },
        text = {
            Column(
                modifier = Modifier
                    .heightIn(max = 440.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("What was it for?") },
                    singleLine = true,
                    enabled = !isSaving,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = amount,
                    onValueChange = { amount = it },
                    label = { Text("Amount") },
                    prefix = { Text("€ ") },
                    singleLine = true,
                    enabled = !isSaving,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )

                if (eachShare != null) {
                    Surface(
                        shape = RoundedCornerShape(14.dp),
                        color = MaterialTheme.colorScheme.primaryContainer,
                        contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                    ) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(horizontal = 14.dp, vertical = 10.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = "Each pays",
                                style = MaterialTheme.typography.bodySmall,
                                modifier = Modifier.weight(1f),
                            )
                            Text(
                                text = formatMoney(eachShare),
                                style = MaterialTheme.typography.titleSmall,
                            )
                        }
                    }
                }

                Text(
                    text = "Split between",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                members.forEach { member ->
                    val checked = selectedMemberIds.contains(member.id)
                    Surface(
                        onClick = {
                            if (!isSaving) {
                                if (checked) {
                                    selectedMemberIds.remove(member.id)
                                } else {
                                    selectedMemberIds.add(member.id)
                                }
                            }
                        },
                        enabled = !isSaving,
                        shape = RoundedCornerShape(14.dp),
                        color = if (checked) {
                            MaterialTheme.colorScheme.secondaryContainer
                        } else {
                            MaterialTheme.colorScheme.surfaceContainerLow
                        },
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Avatar(name = member.username, seed = member.id, size = 32.dp)
                            Spacer(Modifier.width(10.dp))
                            Text(
                                text = member.username,
                                style = MaterialTheme.typography.bodyMedium,
                                modifier = Modifier.weight(1f),
                            )
                            Checkbox(
                                checked = checked,
                                onCheckedChange = null,
                                enabled = !isSaving,
                            )
                        }
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
