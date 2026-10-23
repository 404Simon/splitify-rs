package com.splitify.app.ui.groups

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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.PrimaryScrollableTabRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.debts.DebtRepository
import com.splitify.app.data.dto.GroupDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.invites.InviteRepository
import com.splitify.app.data.maps.MapMarkerRepository
import com.splitify.app.data.recurring.RecurringDebtRepository
import com.splitify.app.data.shoppinglists.ShoppingListRepository
import com.splitify.app.data.transactions.TransactionRepository
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.groups.tabs.BalancesTab
import com.splitify.app.ui.groups.tabs.DebtsTab
import com.splitify.app.ui.groups.tabs.InvitesTab
import com.splitify.app.ui.groups.tabs.PlacesTab
import com.splitify.app.ui.groups.tabs.RecurringDebtsTab
import com.splitify.app.ui.groups.tabs.ShoppingListsTab
import com.splitify.app.ui.groups.tabs.TransactionsTab

private val GroupTabs = listOf(
    "Members",
    "Debts",
    "Transactions",
    "Balances",
    "Shopping",
    "Recurring",
    "Invites",
    "Places",
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GroupDetailScreen(
    viewModel: GroupDetailViewModel,
    debtRepository: DebtRepository,
    transactionRepository: TransactionRepository,
    inviteRepository: InviteRepository,
    shoppingListRepository: ShoppingListRepository,
    recurringDebtRepository: RecurringDebtRepository,
    mapMarkerRepository: MapMarkerRepository,
    authRepository: AuthRepository,
    onBack: () -> Unit,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var selectedTab by rememberSaveable { mutableIntStateOf(0) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = state.group?.name ?: "Group",
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Back",
                        )
                    }
                },
                actions = {
                    IconButton(onClick = viewModel::load) {
                        Icon(Icons.Default.Refresh, contentDescription = "Refresh group")
                    }
                },
            )
        },
    ) { padding ->
        Box(
            modifier = Modifier
                .padding(padding)
                .fillMaxSize(),
        ) {
            when {
                state.isLoading -> LoadingScreen(message = "Loading group…")

                state.error != null -> CenteredDetailError(
                    message = state.error.orEmpty(),
                    onRetry = { viewModel.load() },
                )

                else -> state.group?.let { group ->
                    Column(modifier = Modifier.fillMaxSize()) {
                        GroupHeader(group = group, memberCount = state.members.size)
                        PrimaryScrollableTabRow(selectedTabIndex = selectedTab, edgePadding = 0.dp) {
                            GroupTabs.forEachIndexed { index, title ->
                                Tab(
                                    selected = selectedTab == index,
                                    onClick = { selectedTab = index },
                                    text = { Text(title) },
                                )
                            }
                        }
                        when (selectedTab) {
                            0 -> MembersList(state.members)
                            1 -> DebtsTab(
                                groupId = group.id,
                                members = state.members,
                                debtRepository = debtRepository,
                            )
                            2 -> TransactionsTab(
                                groupId = group.id,
                                members = state.members,
                                currentUserId = state.currentUserId,
                                transactionRepository = transactionRepository,
                            )
                            3 -> BalancesTab(
                                groupId = group.id,
                                transactionRepository = transactionRepository,
                            )
                            4 -> ShoppingListsTab(
                                groupId = group.id,
                                currentUserId = state.currentUserId,
                                shoppingListRepository = shoppingListRepository,
                            )
                            5 -> RecurringDebtsTab(
                                groupId = group.id,
                                members = state.members,
                                recurringDebtRepository = recurringDebtRepository,
                            )
                            6 -> InvitesTab(
                                groupId = group.id,
                                inviteRepository = inviteRepository,
                                authRepository = authRepository,
                            )
                            7 -> PlacesTab(
                                groupId = group.id,
                                currentUserId = state.currentUserId,
                                groupCreatedBy = group.createdBy,
                                mapMarkerRepository = mapMarkerRepository,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun GroupHeader(
    group: GroupDto,
    memberCount: Int,
) {
    Surface(color = MaterialTheme.colorScheme.surfaceVariant) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            val memberLabel = if (memberCount == 1) "member" else "members"
            Text(
                text = "$memberCount $memberLabel",
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = "Created ${group.createdAt.take(10)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun MembersList(members: List<GroupMemberDto>) {
    if (members.isEmpty()) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = "No members found.",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        return
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        items(members, key = { it.id }) { member ->
            Card(modifier = Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Surface(
                        shape = CircleShape,
                        color = MaterialTheme.colorScheme.secondaryContainer,
                        contentColor = MaterialTheme.colorScheme.onSecondaryContainer,
                        modifier = Modifier.size(38.dp),
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Text(
                                text = member.username.take(1).uppercase(),
                                style = MaterialTheme.typography.titleMedium,
                            )
                        }
                    }

                    Spacer(Modifier.width(12.dp))

                    Text(
                        text = member.username,
                        modifier = Modifier.weight(1f),
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )

                    if (member.isCreator) {
                        Surface(
                            shape = MaterialTheme.shapes.small,
                            color = MaterialTheme.colorScheme.primaryContainer,
                            contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                        ) {
                            Text(
                                text = "Creator",
                                modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                                style = MaterialTheme.typography.labelSmall,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun CenteredDetailError(
    message: String,
    onRetry: () -> Unit,
) {
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
            tint = MaterialTheme.colorScheme.error,
            modifier = Modifier.size(40.dp),
        )
        Spacer(Modifier.height(12.dp))
        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            textAlign = TextAlign.Center,
        )
        Spacer(Modifier.height(16.dp))
        Button(onClick = onRetry) {
            Text("Retry")
        }
    }
}
