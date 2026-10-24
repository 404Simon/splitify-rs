package com.splitify.app.ui.groups.tabs

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.dto.DebtRelationshipDto
import com.splitify.app.data.dto.UserBalanceDto
import com.splitify.app.data.transactions.TransactionRepository
import com.splitify.app.ui.components.Avatar
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.IconBubble
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.components.MoneyText
import com.splitify.app.ui.components.Pill
import com.splitify.app.ui.components.SectionHeader
import com.splitify.app.ui.components.SplitifyCard
import com.splitify.app.ui.theme.splitify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class BalancesUiState(
    val balances: List<UserBalanceDto> = emptyList(),
    val isLoading: Boolean = true,
)

class BalancesViewModel(
    groupId: Long,
    transactionRepository: TransactionRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(BalancesUiState())
    val uiState: StateFlow<BalancesUiState> = _uiState.asStateFlow()

    init {
        // Balances are derived locally from the cached snapshot, so this needs no
        // network call and picks up optimistic debts/transactions immediately.
        viewModelScope.launch {
            transactionRepository.observeBalances(groupId).collect { balances ->
                _uiState.update { it.copy(balances = balances, isLoading = false) }
            }
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            transactionRepository: TransactionRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { BalancesViewModel(groupId, transactionRepository) }
        }
    }
}

@Composable
fun BalancesTab(
    groupId: Long,
    currentUserId: Long?,
    transactionRepository: TransactionRepository,
) {
    val viewModel: BalancesViewModel = viewModel(
        key = "balances-$groupId",
        factory = BalancesViewModel.factory(groupId, transactionRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    when {
        state.isLoading && state.balances.isEmpty() ->
            LoadingScreen(message = "Calculating balances…")

        state.balances.isEmpty() -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            EmptyState(
                icon = Icons.Default.CheckCircle,
                title = "All settled up",
                message = "Balances from shared debts and payments will appear here.",
            )
        }

        else -> {
            val mine = state.balances.firstOrNull { it.userId == currentUserId }
            val others = state.balances.filter { it.userId != currentUserId }

            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(start = 20.dp, end = 20.dp, top = 4.dp, bottom = 28.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                if (mine != null) {
                    item(key = "mine") {
                        YourBalanceCard(balance = mine)
                    }
                }
                if (others.isNotEmpty()) {
                    item(key = "everyone-header") {
                        SectionHeader(
                            title = "Group balances",
                            modifier = Modifier.padding(top = 8.dp, bottom = 2.dp),
                        )
                    }
                    items(others, key = { it.userId }) { balance ->
                        BalanceCard(balance = balance)
                    }
                }
            }
        }
    }
}

@Composable
private fun YourBalanceCard(balance: UserBalanceDto) {
    val splitify = MaterialTheme.splitify
    val container: Color
    val content: Color
    val accent: Color
    val icon = when (balance.netType) {
        "Positive" -> {
            container = splitify.positiveContainer
            content = splitify.onPositiveContainer
            accent = splitify.positive
            Icons.Default.CheckCircle
        }

        "Negative" -> {
            container = splitify.negativeContainer
            content = splitify.onNegativeContainer
            accent = splitify.negative
            Icons.Default.Warning
        }

        else -> {
            container = MaterialTheme.colorScheme.surfaceContainerHigh
            content = MaterialTheme.colorScheme.onSurface
            accent = MaterialTheme.colorScheme.onSurfaceVariant
            Icons.Default.Info
        }
    }

    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.extraLarge,
        color = container,
        contentColor = content,
    ) {
        Column(modifier = Modifier.padding(20.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                IconBubble(
                    icon = icon,
                    size = 36.dp,
                    iconSize = 19.dp,
                    containerColor = accent.copy(alpha = 0.14f),
                    contentColor = accent,
                )
                Spacer(Modifier.width(10.dp))
                Text(
                    text = "YOUR BALANCE",
                    style = MaterialTheme.typography.labelSmall,
                    letterSpacing = 1.6.sp,
                    color = content.copy(alpha = 0.7f),
                )
            }
            Spacer(Modifier.height(14.dp))
            MoneyText(
                amount = balance.netAmount,
                style = MaterialTheme.typography.displaySmall,
                color = content,
                showPlus = balance.netType == "Positive",
                isNegative = balance.netType == "Negative",
            )
            Text(
                text = netLabel(balance.netType, isMe = true),
                style = MaterialTheme.typography.bodyMedium,
                color = content.copy(alpha = 0.75f),
            )

            Spacer(Modifier.height(18.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                TotalColumn(
                    label = "Owed to you",
                    amount = balance.totalOwed,
                    content = content,
                    accent = splitify.positive,
                    modifier = Modifier.weight(1f),
                )
                TotalColumn(
                    label = "You owe",
                    amount = balance.totalOwing,
                    content = content,
                    accent = splitify.negative,
                    modifier = Modifier.weight(1f),
                )
            }

            if (balance.relationships.isNotEmpty()) {
                Spacer(Modifier.height(16.dp))
                HorizontalDivider(color = content.copy(alpha = 0.15f))
                Spacer(Modifier.height(6.dp))
                balance.relationships.forEach { relationship ->
                    RelationshipRow(
                        relationship = relationship,
                        ownerName = balance.username,
                        isMe = true,
                        contentColor = content,
                    )
                }
            }
        }
    }
}

@Composable
private fun TotalColumn(
    label: String,
    amount: String,
    content: Color,
    accent: Color,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.medium,
        color = content.copy(alpha = 0.07f),
    ) {
        Column(modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp)) {
            Text(
                text = label,
                style = MaterialTheme.typography.labelSmall,
                color = content.copy(alpha = 0.62f),
            )
            Spacer(Modifier.height(2.dp))
            MoneyText(
                amount = amount,
                style = MaterialTheme.typography.titleMedium,
                color = accent,
            )
        }
    }
}

@Composable
private fun BalanceCard(balance: UserBalanceDto) {
    SplitifyCard(contentPadding = PaddingValues(14.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Avatar(name = balance.username, seed = balance.userId, size = 44.dp)
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = balance.username,
                        style = MaterialTheme.typography.titleMedium,
                    )
                    if (balance.netType == "Neutral") {
                        Spacer(Modifier.width(8.dp))
                        Pill(
                            text = "Settled",
                            containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                            contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                Text(
                    text = netLabel(balance.netType, isMe = false),
                    style = MaterialTheme.typography.bodySmall,
                    color = netColor(balance.netType),
                )
            }
            MoneyText(
                amount = balance.netAmount,
                style = MaterialTheme.typography.titleLarge,
                color = netColor(balance.netType),
                showPlus = balance.netType == "Positive",
                isNegative = balance.netType == "Negative",
            )
        }

        if (balance.relationships.isNotEmpty()) {
            Spacer(Modifier.height(12.dp))
            HorizontalDivider(color = MaterialTheme.splitify.cardBorder)
            balance.relationships.forEach { relationship ->
                RelationshipRow(
                    relationship = relationship,
                    ownerName = balance.username,
                    isMe = false,
                    contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun RelationshipRow(
    relationship: DebtRelationshipDto,
    ownerName: String,
    isMe: Boolean,
    contentColor: Color,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 5.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = relationshipLabel(relationship, ownerName, isMe),
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.bodyMedium,
            color = contentColor,
        )
        MoneyText(
            amount = relationship.amount,
            style = MaterialTheme.typography.bodyMedium,
            color = if (relationship.relationshipType == "Owed") {
                MaterialTheme.splitify.positive
            } else {
                MaterialTheme.splitify.negative
            },
        )
    }
}

private fun netLabel(netType: String, isMe: Boolean): String = when (netType) {
    "Positive" -> if (isMe) "You're owed overall" else "Is owed overall"
    "Negative" -> if (isMe) "You owe overall" else "Owes overall"
    else -> if (isMe) "You're all settled" else "Settled up"
}

@Composable
private fun netColor(netType: String): Color = when (netType) {
    "Positive" -> MaterialTheme.splitify.positive
    "Negative" -> MaterialTheme.splitify.negative
    else -> MaterialTheme.colorScheme.onSurfaceVariant
}

private fun relationshipLabel(
    relationship: DebtRelationshipDto,
    ownerName: String,
    isMe: Boolean,
): String = when (relationship.relationshipType) {
    "Owes" -> if (isMe) {
        "You owe ${relationship.otherUsername}"
    } else {
        "$ownerName owes ${relationship.otherUsername}"
    }

    "Owed" -> if (isMe) {
        "${relationship.otherUsername} owes you"
    } else {
        "${relationship.otherUsername} owes $ownerName"
    }

    else -> relationship.relationshipType
}
