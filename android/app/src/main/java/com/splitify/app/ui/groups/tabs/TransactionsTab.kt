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
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuAnchorType
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
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
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.TransactionDto
import com.splitify.app.data.network.ApiException
import com.splitify.app.data.network.userMessage
import com.splitify.app.data.transactions.TransactionRepository
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

private val TransactionAmountPattern = Regex("""^\d+(\.\d{1,2})?$""")

data class TransactionsUiState(
    val transactions: List<TransactionDto> = emptyList(),
    val isLoading: Boolean = true,
    val error: String? = null,
    val showFormDialog: Boolean = false,
    val editingTransactionId: Long? = null,
    val isSaving: Boolean = false,
    val formError: String? = null,
    val deleteTarget: TransactionDto? = null,
    val isDeleting: Boolean = false,
)

class TransactionsViewModel(
    private val groupId: Long,
    private val transactionRepository: TransactionRepository,
    private val authRepository: AuthRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(TransactionsUiState())
    val uiState: StateFlow<TransactionsUiState> = _uiState.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            transactionRepository.listTransactions(groupId)
                .onSuccess { transactions ->
                    _uiState.update {
                        it.copy(transactions = transactions, isLoading = false)
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

    fun requestDelete(transaction: TransactionDto) {
        _uiState.update { it.copy(deleteTarget = transaction) }
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
            transactionRepository.deleteTransaction(target.id)
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
            transactionRepository: TransactionRepository,
            authRepository: AuthRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer {
                TransactionsViewModel(groupId, transactionRepository, authRepository)
            }
        }
    }
}

@Composable
fun TransactionsTab(
    groupId: Long,
    members: List<GroupMemberDto>,
    currentUserId: Long?,
    transactionRepository: TransactionRepository,
    authRepository: AuthRepository,
) {
    val viewModel: TransactionsViewModel = viewModel(
        key = "transactions-$groupId",
        factory = TransactionsViewModel.factory(groupId, transactionRepository, authRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.load() }

    val recipients = members.filter { it.id != currentUserId }

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.transactions.isEmpty() ->
                LoadingScreen(message = "Loading transactions…")

            state.error != null && state.transactions.isEmpty() -> TabError(
                message = state.error.orEmpty(),
                onRetry = { viewModel.load() },
            )

            state.transactions.isEmpty() ->
                EmptyTransactions(onCreate = { viewModel.openCreate() })

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
                    items(state.transactions, key = { it.id }) { transaction ->
                        TransactionCard(
                            transaction = transaction,
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
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("New transaction") },
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
            title = { Text("Delete transaction") },
            text = {
                Text(
                    "Delete ${transaction.payerUsername} → " +
                        "${transaction.recipientUsername}? This cannot be undone.",
                )
            },
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
private fun TransactionCard(
    transaction: TransactionDto,
    canManage: Boolean,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(start = 16.dp, end = 8.dp, top = 12.dp, bottom = 12.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "${transaction.payerUsername} → " +
                            transaction.recipientUsername,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = transaction.createdAt.take(10),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Text(
                    text = transaction.amount,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.primary,
                )
                if (canManage) {
                    IconButton(onClick = onEdit) {
                        Icon(Icons.Default.Edit, contentDescription = "Edit transaction")
                    }
                    IconButton(onClick = onDelete) {
                        Icon(
                            Icons.Default.Delete,
                            contentDescription = "Delete transaction",
                            tint = MaterialTheme.colorScheme.error,
                        )
                    }
                }
            }

            transaction.description
                ?.takeIf { it.isNotBlank() }
                ?.let { description ->
                    Text(
                        text = description,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
        }
    }
}

@Composable
private fun EmptyTransactions(onCreate: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Default.Info,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(40.dp),
        )
        Spacer(Modifier.width(12.dp))
        Text(text = "No transactions yet", style = MaterialTheme.typography.titleMedium)
        Text(
            text = "Record a payment between group members to settle up.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = 4.dp),
        )
        Button(onClick = onCreate, modifier = Modifier.padding(top = 16.dp)) {
            Text("Add transaction")
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
        title = { Text(if (isEditing) "Edit transaction" else "New transaction") },
        text = {
            Column(
                modifier = Modifier
                    .heightIn(max = 420.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                ExposedDropdownMenuBox(
                    expanded = recipientExpanded,
                    onExpandedChange = { if (!isSaving) recipientExpanded = it },
                ) {
                    OutlinedTextField(
                        value = recipientName,
                        onValueChange = {},
                        readOnly = true,
                        label = { Text("Recipient") },
                        enabled = !isSaving,
                        trailingIcon = {
                            ExposedDropdownMenuDefaults.TrailingIcon(
                                expanded = recipientExpanded,
                            )
                        },
                        modifier = Modifier
                            .menuAnchor(MenuAnchorType.PrimaryNotEditable)
                            .fillMaxWidth(),
                    )
                    ExposedDropdownMenu(
                        expanded = recipientExpanded,
                        onDismissRequest = { recipientExpanded = false },
                    ) {
                        recipients.forEach { member ->
                            DropdownMenuItem(
                                text = { Text(member.username) },
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
                    singleLine = true,
                    enabled = !isSaving,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = description,
                    onValueChange = { description = it },
                    label = { Text("Description (optional)") },
                    enabled = !isSaving,
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
