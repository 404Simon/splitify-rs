package com.splitify.app.ui.groups.tabs

import androidx.compose.foundation.clickable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import com.splitify.app.ui.components.Avatar
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.IconBubble
import com.splitify.app.ui.components.MoneyText
import com.splitify.app.ui.components.Pill
import com.splitify.app.ui.components.SplitifyCard
import com.splitify.app.ui.format.formatMoney
import com.splitify.app.ui.format.splitEvenly
import com.splitify.app.ui.theme.splitify
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
import com.splitify.app.data.dto.CreateRecurringDebtRequest
import com.splitify.app.data.dto.GeneratedInstanceDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.RecurringDebtDto
import com.splitify.app.data.dto.RecurringDebtMemberDto
import com.splitify.app.data.dto.UpdateRecurringDebtRequest
import com.splitify.app.data.network.userMessage
import com.splitify.app.data.recurring.RecurringDebtRepository
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import java.time.LocalDate
import java.time.ZoneOffset
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
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

private data class RecurringDebtSnapshotView(
    val debts: List<RecurringDebtDto>,
    val shares: Map<Long, List<RecurringDebtMemberDto>>,
    val instances: Map<Long, List<GeneratedInstanceDto>>,
    val memberIds: Map<Long, List<Long>>,
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
)

class RecurringDebtsViewModel(
    private val groupId: Long,
    private val recurringDebtRepository: RecurringDebtRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(RecurringDebtsUiState())
    val uiState: StateFlow<RecurringDebtsUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            combine(
                recurringDebtRepository.observeDebts(groupId),
                recurringDebtRepository.observeShares(groupId),
                recurringDebtRepository.observeMemberIds(groupId),
                recurringDebtRepository.observeInstances(groupId),
            ) { debts, shares, memberIds, instances ->
                RecurringDebtSnapshotView(
                    debts = debts,
                    shares = shares,
                    instances = instances,
                    memberIds = memberIds,
                )
            }.collect { snapshot ->
                _uiState.update { state ->
                    state.copy(
                        debts = snapshot.debts,
                        shares = snapshot.shares,
                        instances = snapshot.instances,
                        memberIds = snapshot.memberIds,
                        isLoading = false,
                        error = null,
                    )
                }
            }
        }
    }

    /** Asks the sync engine for a fresh snapshot; cached data stays visible meanwhile. */
    fun refresh() {
        _uiState.update { it.copy(error = null) }
        recurringDebtRepository.refresh()
    }

    fun dismissInfo() {
        _uiState.update { it.copy(infoMessage = null) }
    }

    fun dismissError() {
        _uiState.update { it.copy(error = null) }
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
            parsedEnd != null && !parsedEnd.isAfter(parsedStart) ->
                "End date must be after the start date."
            memberIds.isEmpty() -> "Select at least one member."
            else -> null
        }
        if (validationError != null) {
            _uiState.update { it.copy(formError = validationError) }
            return
        }
        if (_uiState.value.isSaving) return
        _uiState.update { it.copy(isSaving = true, formError = null, error = null) }

        viewModelScope.launch {
            val editingId = _uiState.value.editingDebtId
            val end = trimmedEnd.takeIf { it.isNotEmpty() }
            val result: Result<Unit> = if (editingId != null) {
                recurringDebtRepository.updateRecurringDebt(
                    groupId = groupId,
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
                }
                .onFailure { error ->
                    _uiState.update {
                        it.copy(isSaving = false, formError = error.userMessage())
                    }
                }
        }
    }

    fun toggleActive(debt: RecurringDebtDto) {
        if (debt.id in _uiState.value.togglingIds) return
        val desired = !debt.isActive
        // Flip locally right away; the repository persists the same state into the outbox.
        _uiState.update { state ->
            state.copy(
                debts = state.debts.map { existing ->
                    if (existing.id != debt.id) {
                        existing
                    } else {
                        existing.copy(
                            isActive = desired,
                            status = statusFor(desired, existing.endDate),
                        )
                    }
                },
                togglingIds = state.togglingIds + debt.id,
                error = null,
            )
        }
        viewModelScope.launch {
            recurringDebtRepository.toggleRecurringDebt(groupId, debt.id)
                .onSuccess {
                    _uiState.update { it.copy(togglingIds = it.togglingIds - debt.id) }
                }
                .onFailure { error ->
                    _uiState.update { state ->
                        state.copy(
                            togglingIds = state.togglingIds - debt.id,
                            debts = state.debts.map { if (it.id == debt.id) debt else it },
                            error = error.userMessage(),
                        )
                    }
                }
        }
    }

    fun generateNow(debt: RecurringDebtDto) {
        if (debt.id in _uiState.value.generatingIds) return
        _uiState.update { it.copy(generatingIds = it.generatingIds + debt.id, error = null) }
        viewModelScope.launch {
            recurringDebtRepository.generateRecurringDebt(groupId, debt.id)
                .onSuccess {
                    _uiState.update {
                        it.copy(
                            generatingIds = it.generatingIds - debt.id,
                            infoMessage = "Generation queued. It will appear after the next sync.",
                        )
                    }
                }
                .onFailure { error ->
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
        _uiState.update { it.copy(deleteTarget = null) }
    }

    fun confirmDelete() {
        val target = _uiState.value.deleteTarget ?: return
        // Close the dialog before the (local, near-instant) mutation lands.
        _uiState.update { it.copy(deleteTarget = null, error = null) }
        viewModelScope.launch {
            recurringDebtRepository.deleteRecurringDebt(groupId, target.id)
                .onFailure { error ->
                    _uiState.update { it.copy(error = error.userMessage()) }
                }
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

    /** Mirrors the repository's derived status so the optimistic toggle looks right instantly. */
    private fun statusFor(isActive: Boolean, endDate: String?): String {
        if (!isActive) return "Paused"
        val parsedEnd = endDate?.let { runCatching { LocalDate.parse(it) }.getOrNull() }
        return if (parsedEnd != null && LocalDate.now(ZoneOffset.UTC).isAfter(parsedEnd)) {
            "Expired"
        } else {
            "Active"
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            recurringDebtRepository: RecurringDebtRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer {
                RecurringDebtsViewModel(groupId, recurringDebtRepository)
            }
        }
    }
}

@Composable
fun RecurringDebtsTab(
    groupId: Long,
    members: List<GroupMemberDto>,
    recurringDebtRepository: RecurringDebtRepository,
) {
    val viewModel: RecurringDebtsViewModel = viewModel(
        key = "recurring-debts-$groupId",
        factory = RecurringDebtsViewModel.factory(
            groupId = groupId,
            recurringDebtRepository = recurringDebtRepository,
        ),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.debts.isEmpty() ->
                LoadingScreen(message = "Loading recurring debts…")

            state.error != null && state.debts.isEmpty() -> TabError(
                message = state.error.orEmpty(),
                onRetry = { viewModel.refresh() },
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
                        onRetry = { viewModel.refresh() },
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
                    items(state.debts, key = { it.id }) { debt ->
                        RecurringDebtCard(
                            debt = debt,
                            shares = state.shares[debt.id].orEmpty(),
                            instances = state.instances[debt.id].orEmpty(),
                            memberIds = state.memberIds[debt.id].orEmpty(),
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
private fun RecurringDebtCard(
    debt: RecurringDebtDto,
    shares: List<RecurringDebtMemberDto>,
    instances: List<GeneratedInstanceDto>,
    memberIds: List<Long>,
    expanded: Boolean,
    isToggling: Boolean,
    isGenerating: Boolean,
    onToggleExpanded: () -> Unit,
    onToggleActive: () -> Unit,
    onGenerate: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }

    SplitifyCard(contentPadding = PaddingValues(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            IconBubble(
                icon = Icons.Default.DateRange,
                size = 44.dp,
                containerColor = MaterialTheme.colorScheme.tertiaryContainer,
                contentColor = MaterialTheme.colorScheme.onTertiaryContainer,
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
                    text = "per ${frequencyUnit(debt.frequency)}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            if (debt.isCreator) {
                Box {
                    IconButton(onClick = { menuExpanded = true }) {
                        Icon(
                            imageVector = Icons.Default.MoreVert,
                            contentDescription = "Recurring debt options",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    DropdownMenu(
                        expanded = menuExpanded,
                        onDismissRequest = { menuExpanded = false },
                    ) {
                        DropdownMenuItem(
                            text = { Text(if (debt.isActive) "Pause" else "Resume") },
                            leadingIcon = {
                                Icon(
                                    imageVector = if (debt.isActive) {
                                        Icons.Default.Clear
                                    } else {
                                        Icons.Default.PlayArrow
                                    },
                                    contentDescription = null,
                                )
                            },
                            enabled = !isToggling,
                            onClick = {
                                menuExpanded = false
                                onToggleActive()
                            },
                        )
                        DropdownMenuItem(
                            text = { Text("Generate now") },
                            leadingIcon = { Icon(Icons.Default.Add, contentDescription = null) },
                            enabled = !isGenerating,
                            onClick = {
                                menuExpanded = false
                                onGenerate()
                            },
                        )
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

        Spacer(Modifier.height(12.dp))

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
            Spacer(Modifier.weight(1f))
            if (isToggling || isGenerating) {
                CircularProgressIndicator(
                    modifier = Modifier.size(16.dp),
                    strokeWidth = 2.dp,
                )
            }
        }

        Spacer(Modifier.height(6.dp))

        Text(
            text = if (debt.isActive) "Next on ${debt.nextGenerationDate}" else "Paused",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        debt.endDate?.let { endDate ->
            Text(
                text = "Ends on $endDate",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        TextButton(
            onClick = onToggleExpanded,
            contentPadding = PaddingValues(horizontal = 4.dp, vertical = 4.dp),
        ) {
            Text(if (expanded) "Hide details" else "Show details")
            Spacer(Modifier.width(4.dp))
            Icon(
                imageVector = if (expanded) {
                    Icons.Default.KeyboardArrowUp
                } else {
                    Icons.Default.KeyboardArrowDown
                },
                contentDescription = null,
                modifier = Modifier.size(18.dp),
            )
        }

        if (expanded) {
            HorizontalDivider(color = MaterialTheme.splitify.cardBorder)
            Spacer(Modifier.height(10.dp))
            Text(
                text = if (memberIds.isEmpty()) {
                    "Split between ${shares.size} members"
                } else {
                    "Split between ${memberIds.size} members"
                },
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
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Avatar(name = share.username, seed = share.userId, size = 26.dp)
                        Spacer(Modifier.width(8.dp))
                        Text(
                            text = share.username,
                            style = MaterialTheme.typography.bodySmall,
                            modifier = Modifier.weight(1f),
                        )
                        MoneyText(
                            amount = share.shareAmount,
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
            }

            Spacer(Modifier.height(10.dp))
            Text(
                text = "Recent instances",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (instances.isEmpty()) {
                Text(
                    text = "No instances generated yet.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            } else {
                instances.take(5).forEach { instance ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(vertical = 3.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = instance.debtName,
                            style = MaterialTheme.typography.bodySmall,
                            modifier = Modifier.weight(1f),
                        )
                        MoneyText(
                            amount = instance.amount,
                            style = MaterialTheme.typography.bodySmall,
                        )
                        Spacer(Modifier.width(8.dp))
                        Text(
                            text = instance.createdAt.take(10),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun StatusChip(status: String) {
    val splitify = MaterialTheme.splitify
    val containerColor = when (status) {
        "Active" -> splitify.positiveContainer
        "Paused" -> MaterialTheme.colorScheme.surfaceContainerHigh
        else -> MaterialTheme.colorScheme.secondaryContainer
    }
    val contentColor = when (status) {
        "Active" -> splitify.onPositiveContainer
        "Paused" -> MaterialTheme.colorScheme.onSurfaceVariant
        else -> MaterialTheme.colorScheme.onSecondaryContainer
    }
    Pill(
        text = status,
        containerColor = containerColor,
        contentColor = contentColor,
    )
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
    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        EmptyState(
            icon = Icons.Default.DateRange,
            title = "No recurring debts yet",
            message = "Schedule an expense once and Splitify generates it on repeat.",
            accent = MaterialTheme.colorScheme.tertiary,
            accentContainer = MaterialTheme.colorScheme.tertiaryContainer,
            actionLabel = "Add recurring debt",
            onAction = onCreate,
        )
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
    val eachShare = splitEvenly(amount, selectedMemberIds.size)

    AlertDialog(
        onDismissRequest = onDismiss,
        shape = RoundedCornerShape(28.dp),
        title = { Text(if (isEditing) "Edit recurring debt" else "New recurring debt") },
        text = {
            Column(
                modifier = Modifier
                    .heightIn(max = 460.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("What is it for?") },
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
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = endDate,
                    onValueChange = { endDate = it },
                    label = { Text("End date (optional)") },
                    singleLine = true,
                    enabled = !isSaving,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )

                if (isEditing) {
                    Surface(
                        shape = RoundedCornerShape(14.dp),
                        color = MaterialTheme.colorScheme.surfaceContainerLow,
                    ) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(horizontal = 14.dp, vertical = 8.dp),
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
                            Avatar(name = member.username, seed = member.id, size = 30.dp)
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

private fun frequencyUnit(frequency: String): String = when (frequency) {
    "daily" -> "day"
    "weekly" -> "week"
    "monthly" -> "month"
    "yearly" -> "year"
    else -> frequency
}
