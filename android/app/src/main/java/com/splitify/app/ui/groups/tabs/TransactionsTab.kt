package com.splitify.app.ui.groups.tabs

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
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuAnchorType
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
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
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.TransactionDto
import com.splitify.app.data.network.userMessage
import com.splitify.app.data.transactions.TransactionRepository
import com.splitify.app.ui.components.Avatar
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.components.MoneyText
import com.splitify.app.ui.components.SplitifyCard
import com.splitify.app.ui.format.relativeDate
import com.splitify.app.ui.theme.splitify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

private val TransactionAmountPattern = Regex("""^\d+(\.\d{1,2})?$""")

data class TransactionsUiState(
    val transactions: List<TransactionDto> = emptyList(),
    val isLoading: Boolean = true,
    val actionError: String? = null,
    val showFormDialog: Boolean = false,
    val editingTransactionId: Long? = null,
    val isSaving: Boolean = false,
    val formError: String? = null,
    val deleteTarget: TransactionDto? = null,
)

class TransactionsViewModel(
    private val groupId: Long,
    private val transactionRepository: TransactionRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(TransactionsUiState())
    val uiState: StateFlow<TransactionsUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            transactionRepository.observeTransactions(groupId).collect { transactions ->
                _uiState.update {
                    it.copy(transactions = transactions, isLoading = false)
                }
            }
        }
    }

    fun dismissActionError() {
        _uiState.update { it.copy(actionError = null) }
    }

    fun openCreate() {
        _uiState.update {
            it.copy(showFormDialog = true, editingTransactionId = null, formError = null)
        }
    }

    fun openEdit(transaction: TransactionDto) {
        _uiState.update {
            it.copy(
                showFormDialog = true,
                editingTransactionId = transaction.id,
                formError = null,
            )
        }
    }

    fun dismissForm() {
        if (_uiState.value.isSaving) return
        _uiState.update {
            it.copy(showFormDialog = false, editingTransactionId = null, formError = null)
        }
    }

    fun save(recipientId: Long?, amount: String, description: String) {
        val trimmedAmount = amount.trim()
        val trimmedDescription = description.trim().ifEmpty { null }
        val validationError = when {
            recipientId == null -> "Select a recipient."
            !TransactionAmountPattern.matches(trimmedAmount) ->
                "Enter a valid amount with up to 2 decimals."
            else -> null
        }
        if (validationError != null) {
            _uiState.update { it.copy(formError = validationError) }
            return
        }
        if (_uiState.value.isSaving) return
        val targetRecipientId = recipientId ?: return
        _uiState.update { it.copy(isSaving = true, formError = null) }

        viewModelScope.launch {
            val editingId = _uiState.value.editingTransactionId
            val result: Result<Unit> = if (editingId != null) {
                transactionRepository.updateTransaction(
                    groupId = groupId,
                    transactionId = editingId,
                    recipientId = targetRecipientId,
                    amount = trimmedAmount,
                    description = trimmedDescription,
                )
            } else {
                transactionRepository.createTransaction(
                    groupId = groupId,
                    recipientId = targetRecipientId,
                    amount = trimmedAmount,
                    description = trimmedDescription,
                ).map { }
            }
            result
                .onSuccess {
                    _uiState.update {
                        it.copy(
                            isSaving = false,
                            showFormDialog = false,
                            editingTransactionId = null,
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

    fun requestDelete(transaction: TransactionDto) {
        _uiState.update { it.copy(deleteTarget = transaction) }
    }

    fun dismissDelete() {
        _uiState.update { it.copy(deleteTarget = null) }
    }

    fun confirmDelete() {
        val target = _uiState.value.deleteTarget ?: return
        // Local-first: the cached list is the source of truth, so the dialog closes
        // before the local write finishes and nothing waits for the server.
        _uiState.update { it.copy(deleteTarget = null) }
        viewModelScope.launch {
            transactionRepository.deleteTransaction(groupId, target.id)
                .onFailure { error ->
                    _uiState.update { it.copy(actionError = error.userMessage()) }
                }
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            transactionRepository: TransactionRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { TransactionsViewModel(groupId, transactionRepository) }
        }
    }
}

@Composable
fun TransactionsTab(
    groupId: Long,
    members: List<GroupMemberDto>,
    currentUserId: Long?,
    transactionRepository: TransactionRepository,
) {
    val viewModel: TransactionsViewModel = viewModel(
        key = "transactions-$groupId",
        factory = TransactionsViewModel.factory(groupId, transactionRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val recipients = members.filter { it.id != currentUserId }

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.transactions.isEmpty() ->
                LoadingScreen(message = "Loading payments…")

            state.transactions.isEmpty() -> Box(
                Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                EmptyState(
                    icon = Icons.AutoMirrored.Filled.Send,
                    title = "No payments yet",
                    message = "Record a payment between members whenever someone settles up.",
                    actionLabel = "Add a payment",
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
                    items(state.transactions, key = { it.id }) { transaction ->
                        TransactionCard(
                            transaction = transaction,
                            currentUserId = currentUserId,
                            canManage = currentUserId != null &&
                                transaction.payerId == currentUserId,
                            onEdit = { viewModel.openEdit(transaction) },
                            onDelete = { viewModel.requestDelete(transaction) },
                        )
                    }
                }
            }
        }

        if (!state.isLoading || state.transactions.isNotEmpty()) {
            ExtendedFloatingActionButton(
                onClick = { viewModel.openCreate() },
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(16.dp),
                shape = CircleShape,
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("New payment") },
            )
        }
    }

    if (state.showFormDialog) {
        val editingTransaction = state.transactions
            .firstOrNull { it.id == state.editingTransactionId }
        TransactionFormDialog(
            isEditing = editingTransaction != null,
            recipients = recipients,
            initialRecipientId = editingTransaction?.recipientId
                ?: recipients.firstOrNull()?.id,
            initialAmount = editingTransaction?.amount.orEmpty(),
            initialDescription = editingTransaction?.description.orEmpty(),
            isSaving = state.isSaving,
            error = state.formError,
            onDismiss = { viewModel.dismissForm() },
            onSave = { recipientId, amount, description ->
                viewModel.save(recipientId, amount, description)
            },
        )
    }

    state.deleteTarget?.let { transaction ->
        AlertDialog(
            onDismissRequest = { viewModel.dismissDelete() },
            shape = RoundedCornerShape(28.dp),
            title = { Text("Delete payment") },
            text = {
                Text(
                    "Delete ${transaction.payerUsername} → " +
                        "${transaction.recipientUsername}? This cannot be undone.",
                )
            },
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
private fun TransactionCard(
    transaction: TransactionDto,
    currentUserId: Long?,
    canManage: Boolean,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }

    val title = when {
        transaction.payerId == currentUserId -> "You paid ${transaction.recipientUsername}"
        transaction.recipientId == currentUserId -> "${transaction.payerUsername} paid you"
        else -> "${transaction.payerUsername} paid ${transaction.recipientUsername}"
    }

    SplitifyCard(contentPadding = PaddingValues(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Avatar(
                        name = transaction.payerUsername,
                        seed = transaction.payerId,
                        size = 38.dp,
                    )
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier
                            .padding(horizontal = 3.dp)
                            .size(15.dp),
                    )
                    Avatar(
                        name = transaction.recipientUsername,
                        seed = transaction.recipientId,
                        size = 38.dp,
                    )
                }
            }
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = relativeDate(transaction.createdAt) ?: transaction.createdAt.take(10),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            MoneyText(
                amount = transaction.amount,
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.primary,
            )
            if (canManage) {
                Box {
                    IconButton(onClick = { menuExpanded = true }) {
                        Icon(
                            imageVector = Icons.Default.MoreVert,
                            contentDescription = "Payment options",
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

        transaction.description
            ?.takeIf { it.isNotBlank() }
            ?.let { description ->
                Spacer(Modifier.size(10.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.Send,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(14.dp),
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = description,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TransactionFormDialog(
    isEditing: Boolean,
    recipients: List<GroupMemberDto>,
    initialRecipientId: Long?,
    initialAmount: String,
    initialDescription: String,
    isSaving: Boolean,
    error: String?,
    onDismiss: () -> Unit,
    onSave: (Long?, String, String) -> Unit,
) {
    var recipientId by remember { mutableStateOf(initialRecipientId) }
    var amount by rememberSaveable { mutableStateOf(initialAmount) }
    var description by rememberSaveable { mutableStateOf(initialDescription) }
    var recipientExpanded by remember { mutableStateOf(false) }

    var recipientName by remember {
        mutableStateOf(recipients.firstOrNull { it.id == initialRecipientId }?.username.orEmpty())
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        shape = RoundedCornerShape(28.dp),
        title = { Text(if (isEditing) "Edit payment" else "New payment") },
        text = {
            Column(
                modifier = Modifier
                    .heightIn(max = 440.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                ExposedDropdownMenuBox(
                    expanded = recipientExpanded,
                    onExpandedChange = { if (!isSaving) recipientExpanded = it },
                ) {
                    OutlinedTextField(
                        value = recipientName,
                        onValueChange = {},
                        readOnly = true,
                        label = { Text("Paid to") },
                        enabled = !isSaving,
                        shape = RoundedCornerShape(14.dp),
                        trailingIcon = {
                            ExposedDropdownMenuDefaults.TrailingIcon(
                                expanded = recipientExpanded,
                            )
                        },
                        modifier = Modifier
                            .menuAnchor(ExposedDropdownMenuAnchorType.PrimaryNotEditable)
                            .fillMaxWidth(),
                    )
                    ExposedDropdownMenu(
                        expanded = recipientExpanded,
                        onDismissRequest = { recipientExpanded = false },
                    ) {
                        recipients.forEach { member ->
                            DropdownMenuItem(
                                text = { Text(member.username) },
                                leadingIcon = {
                                    Avatar(
                                        name = member.username,
                                        seed = member.id,
                                        size = 28.dp,
                                    )
                                },
                                onClick = {
                                    recipientId = member.id
                                    recipientName = member.username
                                    recipientExpanded = false
                                },
                            )
                        }
                    }
                }
                if (recipients.isEmpty()) {
                    Text(
                        text = "There are no other members to pay.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
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
                OutlinedTextField(
                    value = description,
                    onValueChange = { description = it },
                    label = { Text("Note (optional)") },
                    enabled = !isSaving,
                    shape = RoundedCornerShape(14.dp),
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
                onClick = { onSave(recipientId, amount, description) },
                enabled = !isSaving &&
                    recipientId != null &&
                    amount.isNotBlank(),
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
