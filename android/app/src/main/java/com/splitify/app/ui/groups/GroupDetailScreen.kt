package com.splitify.app.ui.groups

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Place
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.ShoppingCart
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
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
import com.splitify.app.ui.components.Avatar
import com.splitify.app.ui.components.AvatarStack
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.IconBubble
import com.splitify.app.ui.components.LightStatusBarIcons
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.components.Pill
import com.splitify.app.ui.components.PillTab
import com.splitify.app.ui.components.PillTabRow
import com.splitify.app.ui.components.SplitifyCard
import com.splitify.app.ui.format.formatDate
import com.splitify.app.ui.groups.tabs.BalancesTab
import com.splitify.app.ui.groups.tabs.DebtsTab
import com.splitify.app.ui.groups.tabs.InvitesTab
import com.splitify.app.ui.groups.tabs.PlacesTab
import com.splitify.app.ui.groups.tabs.RecurringDebtsTab
import com.splitify.app.ui.groups.tabs.ShoppingListsTab
import com.splitify.app.ui.groups.tabs.TransactionsTab
import com.splitify.app.ui.theme.splitify

private val GroupTabs = listOf(
    PillTab("Members", Icons.Default.Person),
    PillTab("Debts", Icons.AutoMirrored.Filled.List),
    PillTab("Payments", Icons.AutoMirrored.Filled.Send),
    PillTab("Balances", Icons.Default.CheckCircle),
    PillTab("Shopping", Icons.Default.ShoppingCart),
    PillTab("Recurring", Icons.Default.DateRange),
    PillTab("Invites", Icons.Default.Share),
    PillTab("Places", Icons.Default.Place),
)

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

    // The gradient app bar is dark in both themes, so force light status icons.
    LightStatusBarIcons()

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            GroupTopBar(
                group = state.group,
                onBack = onBack,
                onRefresh = viewModel::load,
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
                        GroupMeta(members = state.members, group = group)
                        PillTabRow(
                            tabs = GroupTabs,
                            selectedIndex = selectedTab,
                            onSelected = { selectedTab = it },
                            contentPadding = PaddingValues(
                                start = 20.dp,
                                end = 20.dp,
                                top = 2.dp,
                                bottom = 10.dp,
                            ),
                        )
                        Box(modifier = Modifier.weight(1f)) {
                            when (selectedTab) {
                                0 -> MembersList(
                                    members = state.members,
                                    currentUserId = state.currentUserId,
                                )

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
                                    currentUserId = state.currentUserId,
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
}

@Composable
private fun GroupTopBar(
    group: GroupDto?,
    onBack: () -> Unit,
    onRefresh: () -> Unit,
) {
    val gradient = MaterialTheme.splitify.heroGradient
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clipToBounds()
            .background(Brush.horizontalGradient(gradient)),
    ) {
        Box(
            modifier = Modifier
                .offset(x = 270.dp, y = (-60).dp)
                .size(150.dp)
                .background(Color.White.copy(alpha = 0.08f), CircleShape),
        )
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .windowInsetsPadding(WindowInsets.statusBars)
                .padding(start = 6.dp, end = 6.dp, top = 6.dp, bottom = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = onBack) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                    contentDescription = "Back",
                    tint = Color.White,
                )
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "GROUP",
                    style = MaterialTheme.typography.labelSmall,
                    letterSpacing = 1.8.sp,
                    color = Color.White.copy(alpha = 0.65f),
                )
                Text(
                    text = group?.name ?: "Group",
                    style = MaterialTheme.typography.titleLarge,
                    color = Color.White,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            IconButton(onClick = onRefresh) {
                Icon(
                    imageVector = Icons.Default.Refresh,
                    contentDescription = "Refresh group",
                    tint = Color.White,
                )
            }
        }
    }
}

@Composable
private fun GroupMeta(
    members: List<GroupMemberDto>,
    group: GroupDto,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 20.dp, end = 20.dp, top = 14.dp, bottom = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        AvatarStack(
            people = members.map { it.id to it.username },
            size = 30.dp,
            max = 5,
            ringColor = MaterialTheme.colorScheme.background,
        )
        Spacer(Modifier.width(14.dp))
        Column {
            Text(
                text = if (members.size == 1) "1 member" else "${members.size} members",
                style = MaterialTheme.typography.titleSmall,
            )
            formatDate(group.createdAt)?.let { created ->
                Text(
                    text = "Created $created",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun MembersList(
    members: List<GroupMemberDto>,
    currentUserId: Long?,
) {
    if (members.isEmpty()) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            EmptyState(
                icon = Icons.Default.Person,
                title = "No members yet",
                message = "Invite someone from the Invites tab to get started.",
            )
        }
        return
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 20.dp, end = 20.dp, top = 4.dp, bottom = 28.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        items(members, key = { it.id }) { member ->
            SplitifyCard(contentPadding = PaddingValues(14.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Avatar(name = member.username, seed = member.id, size = 46.dp)
                    Spacer(Modifier.width(12.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Text(
                                text = member.username,
                                style = MaterialTheme.typography.titleMedium,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                modifier = Modifier.weight(1f, fill = false),
                            )
                            if (member.id == currentUserId) {
                                Spacer(Modifier.width(8.dp))
                                Pill(
                                    text = "You",
                                    containerColor = MaterialTheme.colorScheme.secondaryContainer,
                                    contentColor = MaterialTheme.colorScheme.onSecondaryContainer,
                                )
                            }
                        }
                        Spacer(Modifier.height(2.dp))
                        Text(
                            text = if (member.isCreator) "Group creator" else "Member",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    if (member.isCreator) {
                        IconBubble(
                            icon = Icons.Default.Star,
                            size = 36.dp,
                            iconSize = 19.dp,
                            containerColor = MaterialTheme.colorScheme.tertiaryContainer,
                            contentColor = MaterialTheme.colorScheme.onTertiaryContainer,
                        )
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
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        EmptyState(
            icon = Icons.Default.Warning,
            title = "Something went wrong",
            message = message,
            accent = MaterialTheme.colorScheme.error,
            accentContainer = MaterialTheme.colorScheme.errorContainer,
            actionLabel = "Try again",
            onAction = onRetry,
            modifier = Modifier.padding(horizontal = 16.dp),
        )
    }
}
