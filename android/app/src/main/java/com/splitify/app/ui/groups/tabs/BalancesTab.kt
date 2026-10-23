package com.splitify.app.ui.groups.tabs

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Card
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import com.splitify.app.data.dto.DebtRelationshipDto
import com.splitify.app.data.dto.UserBalanceDto
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

data class BalancesUiState(
    val balances: List<UserBalanceDto> = emptyList(),
    val isLoading: Boolean = true,
    val error: String? = null,
)

class BalancesViewModel(
    private val groupId: Long,
    private val transactionRepository: TransactionRepository,
    private val authRepository: AuthRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(BalancesUiState())
    val uiState: StateFlow<BalancesUiState> = _uiState.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            transactionRepository.getBalances(groupId)
                .onSuccess { balances ->
                    _uiState.update { it.copy(balances = balances, isLoading = false) }
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update { it.copy(isLoading = false, error = error.userMessage()) }
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
            initializer { BalancesViewModel(groupId, transactionRepository, authRepository) }
        }
    }
}

@Composable
fun BalancesTab(
    groupId: Long,
    transactionRepository: TransactionRepository,
    authRepository: AuthRepository,
) {
    val viewModel: BalancesViewModel = viewModel(
        key = "balances-$groupId",
        factory = BalancesViewModel.factory(groupId, transactionRepository, authRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.load() }

    when {
        state.isLoading && state.balances.isEmpty() ->
            LoadingScreen(message = "Calculating balances…")

        state.error != null && state.balances.isEmpty() -> Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(24.dp),
            verticalArrangement = Arrangement.Center,
        ) {
            ErrorBanner(message = state.error.orEmpty(), onRetry = { viewModel.load() })
        }

        state.balances.isEmpty() -> EmptyBalances()

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
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                items(state.balances, key = { it.userId }) { balance ->
                    BalanceCard(balance = balance)
                }
            }
        }
    }
}

@Composable
private fun BalanceCard(balance: UserBalanceDto) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Surface(
                    shape = MaterialTheme.shapes.small,
                    color = MaterialTheme.colorScheme.secondaryContainer,
                    contentColor = MaterialTheme.colorScheme.onSecondaryContainer,
                    modifier = Modifier.size(36.dp),
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Text(
                            text = balance.username.take(1).uppercase(),
                            style = MaterialTheme.typography.titleMedium,
                        )
                    }
                }
                Spacer(Modifier.width(12.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = balance.username,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = netLabel(balance.netType),
                        style = MaterialTheme.typography.bodySmall,
                        color = netColor(balance.netType),
                    )
                }
                Text(
                    text = balance.netAmount,
                    style = MaterialTheme.typography.titleLarge,
                    color = netColor(balance.netType),
                )
            }

            Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                LabeledAmount(label = "Total owed", amount = balance.totalOwed)
                LabeledAmount(label = "Total owing", amount = balance.totalOwing)
            }

            if (balance.relationships.isNotEmpty()) {
                HorizontalDivider()
                balance.relationships.forEach { relationship ->
                    RelationshipRow(relationship = relationship)
                }
            }
        }
    }
}

@Composable
private fun LabeledAmount(label: String, amount: String) {
    Column {
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(text = amount, style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
private fun RelationshipRow(relationship: DebtRelationshipDto) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = relationshipLabel(relationship),
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.bodyMedium,
        )
        Text(
            text = relationship.amount,
            style = MaterialTheme.typography.bodyMedium,
            color = if (relationship.relationshipType == "Owed") {
                MaterialTheme.colorScheme.primary
            } else {
                MaterialTheme.colorScheme.error
            },
        )
    }
}

@Composable
private fun EmptyBalances() {
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
        Text(text = "All settled up", style = MaterialTheme.typography.titleMedium)
        Text(
            text = "Balances from shared debts and transactions will appear here.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = 4.dp),
        )
    }
}

private fun netLabel(netType: String): String = when (netType) {
    "Positive" -> "Is owed overall"
    "Negative" -> "Owes overall"
    else -> "Settled up"
}

@Composable
private fun netColor(netType: String) = when (netType) {
    "Positive" -> MaterialTheme.colorScheme.primary
    "Negative" -> MaterialTheme.colorScheme.error
    else -> MaterialTheme.colorScheme.onSurfaceVariant
}

private fun relationshipLabel(relationship: DebtRelationshipDto): String =
    when (relationship.relationshipType) {
        "Owes" -> "You owe ${relationship.otherUsername}"
        "Owed" -> "${relationship.otherUsername} owes you"
        else -> relationship.relationshipType
    }
