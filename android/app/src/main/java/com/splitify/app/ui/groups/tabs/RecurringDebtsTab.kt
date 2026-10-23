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
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
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
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.dto.CreateRecurringDebtRequest
import com.splitify.app.data.dto.GeneratedInstanceDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.RecurringDebtDto
import com.splitify.app.data.dto.RecurringDebtMemberDto
import com.splitify.app.data.dto.UpdateRecurringDebtRequest
import com.splitify.app.data.network.ApiException
import com.splitify.app.data.network.userMessage
import com.splitify.app.data.recurring.RecurringDebtRepository
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import java.time.LocalDate
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

private val AmountPattern = Regex("""^\d+(\.\d{1,2})?$""")
private val DatePattern = Regex("""^\d{4}-\d{2}-\d{2}$""")
private val Frequencies = listOf(
    "daily" to "Daily",
    "weekly" to "Weekly",
    "monthly" to "Monthly",
    "yearly" to "Yearly",
)
private val FrequencyValues = Frequencies.map { it.first }.toSet()

private data class RecurringDebtDetails(
    val shares: List<RecurringDebtMemberDto>,
    val instances: List<GeneratedInstanceDto>,
    val memberIds: List<Long>,
)

data class RecurringDebtsUiState(
    val debts: List<RecurringDebtDto> = emptyList(),
    val shares: Map<Long, List<RecurringDebtMemberDto>> = emptyMap(),
    val instances: Map<Long, List<GeneratedInstanceDto>> = emptyMap(),
    val memberIds: Map<Long, List<Long>> = emptyMap(),
    val expandedIds: Set<Long> = emptySet(),
    val isLoading: Boolean = true,
    val error: String? = null,
    val infoMessage: String? = null,
    val showFormDialog: Boolean = false,
    val editingDebtId: Long? = null,
    val isSaving: Boolean = false,
    val formError: String? = null,
    val togglingIds: Set<Long> = emptySet(),
    val generatingIds: Set<Long> = emptySet(),
    val deleteTarget: RecurringDebtDto? = null,
    val isDeleting: Boolean = false,
)

class RecurringDebtsViewModel(
    private val groupId: Long,
    private val recurringDebtRepository: RecurringDebtRepository,
    private val authRepository: AuthRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(RecurringDebtsUiState())
    val uiState: StateFlow<RecurringDebtsUiState> = _uiState.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            recurringDebtRepository.listRecurringDebts(groupId)
                .onSuccess { debts ->
                    val details = coroutineScope {
                        debts.map { debt ->
                            async {
                                debt.id to RecurringDebtDetails(
                                    shares = recurringDebtRepository
                                        .getRecurringDebtShares(debt.id)
                                        .getOrDefault(emptyList()),
                                    instances = recurringDebtRepository
                                        .listInstances(debt.id)
                                        .getOrDefault(emptyList()),
                                    memberIds = recurringDebtRepository
                                        .getRecurringDebtMembers(debt.id)
                                        .getOrDefault(emptyList()),
                                )
                            }
                        }.awaitAll().toMap()
                    }
                    _uiState.update {
                        it.copy(
                            debts = debts,
                            shares = details.mapValues { entry -> entry.value.shares },
                            instances = details.mapValues { entry -> entry.value.instances },
                            memberIds = details.mapValues { entry -> entry.value.memberIds },
                            isLoading = false,
                        )
                    }
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update { it.copy(isLoading = false, error = error.userMessage()) }
                }
        }
    }

    fun dismissInfo() {
        _uiState.update { it.copy(infoMessage = null) }
    }

    fun toggleExpanded(debtId: Long) {
        _uiState.update { state ->
            val expanded = if (debtId in state.expandedIds) {
                state.expandedIds - debtId
            } else {
                state.expandedIds + debtId
            }
            state.copy(expandedIds = expanded)
        }
    }

    fun openCreate() {
        _uiState.update {
            it.copy(showFormDialog = true, editingDebtId = null, formError = null)
        }
    }

    fun openEdit(debt: RecurringDebtDto) {
        _uiState.update {
            it.copy(showFormDialog = true, editingDebtId = debt.id, formError = null)
        }
    }

    fun dismissForm() {
        if (_uiState.value.isSaving) return
        _uiState.update { it.copy(showFormDialog = false, editingDebtId = null, formError = null) }
    }

    fun save(
        name: String,
        amount: String,
        frequency: String,
        startDate: String,
        endDate: String,
        isActive: Boolean,
        memberIds: List<Long>,
    ) {
        val trimmedName = name.trim()
        val trimmedAmount = amount.trim()
        val trimmedStart = startDate.trim()
        val trimmedEnd = endDate.trim()
        val parsedStart = parseDate(trimmedStart)
        val parsedEnd = trimmedEnd.takeIf { it.isNotEmpty() }?.let(::parseDate)
        val validationError = when {
            trimmedName.isEmpty() -> "Enter a name."
            !AmountPattern.matches(trimmedAmount) -> "Enter a valid amount with up to 2 decimals."
            frequency !in FrequencyValues -> "Choose a frequency."
            parsedStart == null -> "Enter a valid start date (YYYY-MM-DD)."
            trimmedEnd.isNotEmpty() && parsedEnd == null ->
                "Enter a valid end date (YYYY-MM-DD)."
            parsedEnd != null && parsedStart != null && !parsedEnd.isAfter(parsedStart) ->
                "End date must be after the start date."
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
            val end = trimmedEnd.takeIf { it.isNotEmpty() }
            val result: Result<Unit> = if (editingId != null) {
                recurringDebtRepository.updateRecurringDebt(
                    recurringDebtId = editingId,
                    request = UpdateRecurringDebtRequest(
                        name = trimmedName,
                        amount = trimmedAmount,
                        frequency = frequency,
                        endDate = end,
                        isActive = isActive,
                        memberIds = memberIds,
                    ),
                )
            } else {
                recurringDebtRepository.createRecurringDebt(
                    groupId = groupId,
                    request = CreateRecurringDebtRequest(
                        name = trimmedName,
                        amount = trimmedAmount,
                        frequency = frequency,
                        startDate = trimmedStart,
                        endDate = end,
                        memberIds = memberIds,
                    ),
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

    fun toggleActive(debt: RecurringDebtDto) {
        if (debt.id in _uiState.value.togglingIds) return
        _uiState.update { it.copy(togglingIds = it.togglingIds + debt.id) }
        viewModelScope.launch {
            recurringDebtRepository.toggleRecurringDebt(debt.id)
                .onSuccess {
                    _uiState.update { it.copy(togglingIds = it.togglingIds - debt.id) }
                    load()
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update {
                        it.copy(
                            togglingIds = it.togglingIds - debt.id,
                            error = error.userMessage(),
                        )
                    }
                }
        }
    }

    fun generateNow(debt: RecurringDebtDto) {
        if (debt.id in _uiState.value.generatingIds) return
        _uiState.update { it.copy(generatingIds = it.generatingIds + debt.id) }
        viewModelScope.launch {
            recurringDebtRepository.generateRecurringDebt(debt.id)
                .onSuccess { sharedDebtId ->
                    _uiState.update {
                        it.copy(
                            generatingIds = it.generatingIds - debt.id,
                            infoMessage = "Generated shared debt #$sharedDebtId.",
                        )
                    }
                    load()
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update {
                        it.copy(
                            generatingIds = it.generatingIds - debt.id,
                            error = error.userMessage(),
                        )
                    }
                }
        }
    }

    fun requestDelete(debt: RecurringDebtDto) {
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
            recurringDebtRepository.deleteRecurringDebt(target.id)
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

    private fun parseDate(value: String): LocalDate? {
        if (!DatePattern.matches(value)) return null
        return try {
            LocalDate.parse(value)
        } catch (_: Exception) {
            null
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            recurringDebtRepository: RecurringDebtRepository,
            authRepository: AuthRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer {
                RecurringDebtsViewModel(groupId, recurringDebtRepository, authRepository)
            }
        }
    }
}

@Composable
fun RecurringDebtsTab(
    groupId: Long,
    members: List<GroupMemberDto>,
    recurringDebtRepository: RecurringDebtRepository,
    authRepository: AuthRepository,
) {
    val viewModel: RecurringDebtsViewModel = viewModel(
        key = "recurring-debts-$groupId",
        factory = RecurringDebtsViewModel.factory(
            groupId = groupId,
            recurringDebtRepository = recurringDebtRepository,
            authRepository = authRepository,
        ),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.load() }

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.debts.isEmpty() ->
                LoadingScreen(message = "Loading recurring debts…")

            state.error != null && state.debts.isEmpty() -> TabError(
                message = state.error.orEmpty(),
                onRetry = { viewModel.load() },
            )

            state.debts.isEmpty() -> EmptyRecurringDebts(onCreate = { viewModel.openCreate() })

            else -> Column(modifier = Modifier.fillMaxSize()) {
                state.infoMessage?.let { message ->
                    InfoBanner(
                        message = message,
                        onDismiss = { viewModel.dismissInfo() },
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                    )
                }
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
                        RecurringDebtCard(
                            debt = debt,
                            shares = state.shares[debt.id].orEmpty(),
                            instances = state.instances[debt.id].orEmpty(),
                            expanded = debt.id in state.expandedIds,
                            isToggling = debt.id in state.togglingIds,
                            isGenerating = debt.id in state.generatingIds,
                            onToggleExpanded = { viewModel.toggleExpanded(debt.id) },
                            onToggleActive = { viewModel.toggleActive(debt) },
                            onGenerate = { viewModel.generateNow(debt) },
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
                text = { Text("New recurring") },
            )
        }
    }

    if (state.showFormDialog) {
        val editingDebt = state.debts.firstOrNull { it.id == state.editingDebtId }
        val initialMemberIds = editingDebt
            ?.let { debt -> state.memberIds[debt.id] }
            ?.takeIf { it.isNotEmpty() }
            ?: editingDebt
                ?.let { debt -> state.shares[debt.id]?.map { it.userId } }
                ?.takeIf { it.isNotEmpty() }
            ?: members.map { it.id }
        RecurringDebtFormDialog(
            isEditing = editingDebt != null,
            initialName = editingDebt?.name.orEmpty(),
            initialAmount = editingDebt?.amount.orEmpty(),
            initialFrequency = editingDebt?.frequency ?: Frequencies.first().first,
            initialStartDate = editingDebt?.startDate.orEmpty(),
            initialEndDate = editingDebt?.endDate.orEmpty(),
            initialActive = editingDebt?.isActive ?: true,
            initialMemberIds = initialMemberIds,
            members = members,
            isSaving = state.isSaving,
            error = state.formError,
            onDismiss = { viewModel.dismissForm() },
            onSave = { name, amount, frequency, startDate, endDate, isActive, memberIds ->
                viewModel.save(
                    name = name,
                    amount = amount,
                    frequency = frequency,
                    startDate = startDate,
                    endDate = endDate,
                    isActive = isActive,
                    memberIds = memberIds,
                )
            },
        )
    }

    state.deleteTarget?.let { debt ->
        AlertDialog(
            onDismissRequest = { viewModel.dismissDelete() },
            title = { Text("Delete recurring debt") },
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
private fun RecurringDebtCard(
    debt: RecurringDebtDto,
    shares: List<RecurringDebtMemberDto>,
    instances: List<GeneratedInstanceDto>,
    expanded: Boolean,
    isToggling: Boolean,
    isGenerating: Boolean,
    onToggleExpanded: () -> Unit,
    onToggleActive: () -> Unit,
    onGenerate: () -> Unit,
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
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                StatusChip(status = debt.status)
                Text(
                    text = frequencyLabel(debt.frequency),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Text(
                text = "Next generation ${debt.nextGenerationDate}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            debt.endDate?.let { endDate ->
                Text(
                    text = "Ends $endDate",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(2.dp),
            ) {
                TextButton(onClick = onToggleExpanded) {
                    Text(if (expanded) "Hide details" else "Details")
                }
                if (debt.isCreator) {
                    TextButton(onClick = onToggleActive, enabled = !isToggling) {
                        if (isToggling) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                strokeWidth = 2.dp,
                            )
                        } else {
                            Text(if (debt.isActive) "Pause" else "Resume")
                        }
                    }
                    TextButton(onClick = onGenerate, enabled = !isGenerating) {
                        if (isGenerating) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                strokeWidth = 2.dp,
                            )
                        } else {
                            Text("Generate now")
                        }
                    }
                    IconButton(onClick = onEdit) {
                        Icon(Icons.Default.Edit, contentDescription = "Edit recurring debt")
                    }
                    IconButton(onClick = onDelete) {
                        Icon(
                            Icons.Default.Delete,
                            contentDescription = "Delete recurring debt",
                            tint = MaterialTheme.colorScheme.error,
                        )
                    }
                }
            }

            if (expanded) {
                Text(
                    text = "Shares",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                if (shares.isEmpty()) {
                    Text(
                        text = "No shares available.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                } else {
                    shares.forEach { share ->
                        Text(
                            text = "${share.username} · ${share.shareAmount}",
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }

                Text(
                    text = "Recent instances",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 4.dp),
                )
                if (instances.isEmpty()) {
                    Text(
                        text = "No instances generated yet.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                } else {
                    instances.take(5).forEach { instance ->
                        Text(
                            text = "#${instance.sharedDebtId} · ${instance.debtName} · " +
                                "${instance.amount} · " +
                                instance.createdAt.take(16).replace('T', ' '),
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun StatusChip(status: String) {
    val containerColor = when (status) {
        "Active" -> MaterialTheme.colorScheme.primaryContainer
        "Paused" -> MaterialTheme.colorScheme.secondaryContainer
        else -> MaterialTheme.colorScheme.surfaceVariant
    }
    val contentColor = when (status) {
        "Active" -> MaterialTheme.colorScheme.onPrimaryContainer
        "Paused" -> MaterialTheme.colorScheme.onSecondaryContainer
        else -> MaterialTheme.colorScheme.onSurfaceVariant
    }
    Surface(
        shape = MaterialTheme.shapes.small,
        color = containerColor,
        contentColor = contentColor,
    ) {
        Text(
            text = status,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
            style = MaterialTheme.typography.labelSmall,
        )
    }
}

@Composable
private fun InfoBanner(
    message: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.primaryContainer,
        contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
    ) {
        Row(
            modifier = Modifier.padding(start = 16.dp, end = 8.dp, top = 4.dp, bottom = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = message,
                modifier = Modifier.weight(1f),
                style = MaterialTheme.typography.bodyMedium,
            )
            TextButton(onClick = onDismiss) {
                Text("Dismiss")
            }
        }
    }
}

@Composable
private fun EmptyRecurringDebts(onCreate: () -> Unit) {
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
        Spacer(Modifier.height(12.dp))
        Text(text = "No recurring debts yet", style = MaterialTheme.typography.titleMedium)
        Text(
            text = "Schedule a debt to be generated automatically on a regular basis.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = 4.dp),
        )
        Button(onClick = onCreate, modifier = Modifier.padding(top = 16.dp)) {
            Text("Add recurring debt")
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
private fun RecurringDebtFormDialog(
    isEditing: Boolean,
    initialName: String,
    initialAmount: String,
    initialFrequency: String,
    initialStartDate: String,
    initialEndDate: String,
    initialActive: Boolean,
    initialMemberIds: List<Long>,
    members: List<GroupMemberDto>,
    isSaving: Boolean,
    error: String?,
    onDismiss: () -> Unit,
    onSave: (String, String, String, String, String, Boolean, List<Long>) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf(initialName) }
    var amount by rememberSaveable { mutableStateOf(initialAmount) }
    var frequency by rememberSaveable { mutableStateOf(initialFrequency) }
    var startDate by rememberSaveable { mutableStateOf(initialStartDate) }
    var endDate by rememberSaveable { mutableStateOf(initialEndDate) }
    var isActive by rememberSaveable { mutableStateOf(initialActive) }
    val selectedMemberIds = remember {
        mutableStateListOf<Long>().apply { addAll(initialMemberIds) }
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(if (isEditing) "Edit recurring debt" else "New recurring debt") },
        text = {
            Column(
                modifier = Modifier
                    .heightIn(max = 460.dp)
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
                    text = "Frequency",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                    Frequencies.forEachIndexed { index, (value, label) ->
                        SegmentedButton(
                            selected = frequency == value,
                            onClick = { frequency = value },
                            enabled = !isSaving,
                            shape = SegmentedButtonDefaults.itemShape(
                                index = index,
                                count = Frequencies.size,
                            ),
                            label = { Text(label) },
                        )
                    }
                }
                OutlinedTextField(
                    value = startDate,
                    onValueChange = { startDate = it },
                    label = { Text("Start date (YYYY-MM-DD)") },
                    singleLine = true,
                    enabled = !isSaving && !isEditing,
                    supportingText = if (isEditing) {
                        { Text("The start date cannot be changed.") }
                    } else {
                        null
                    },
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = endDate,
                    onValueChange = { endDate = it },
                    label = { Text("End date (optional, YYYY-MM-DD)") },
                    singleLine = true,
                    enabled = !isSaving,
                    modifier = Modifier.fillMaxWidth(),
                )

                if (isEditing) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(text = "Active", style = MaterialTheme.typography.bodyLarge)
                            Text(
                                text = "Paused schedules do not generate debts.",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        Spacer(Modifier.width(12.dp))
                        Switch(
                            checked = isActive,
                            onCheckedChange = { isActive = it },
                            enabled = !isSaving,
                        )
                    }
                }

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
                onClick = {
                    onSave(
                        name,
                        amount,
                        frequency,
                        startDate,
                        endDate,
                        isActive,
                        selectedMemberIds.toList(),
                    )
                },
                enabled = !isSaving &&
                    name.isNotBlank() &&
                    amount.isNotBlank() &&
                    startDate.isNotBlank() &&
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

private fun frequencyLabel(frequency: String): String =
    Frequencies.firstOrNull { it.first == frequency }?.second ?: frequency
