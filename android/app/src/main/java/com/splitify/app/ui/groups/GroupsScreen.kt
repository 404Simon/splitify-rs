package com.splitify.app.ui.groups

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ExitToApp
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.splitify.app.data.dto.GroupSummaryDto
import com.splitify.app.ui.components.Avatar
import com.splitify.app.ui.components.BrandMark
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.components.Pill
import com.splitify.app.ui.components.SectionHeader
import com.splitify.app.ui.components.SplitifyCard
import com.splitify.app.ui.format.relativeDate
import com.splitify.app.ui.theme.splitify
import java.time.LocalTime

@Composable
fun GroupsScreen(
    viewModel: GroupsViewModel,
    username: String?,
    onOpenGroup: (Long) -> Unit,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(state.acceptedGroupId) {
        state.acceptedGroupId?.let { groupId ->
            onOpenGroup(groupId)
            viewModel.consumeAcceptedGroup()
        }
    }

    val totalMembers = state.groups.sumOf { it.memberCount }

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = { viewModel.setCreateDialogVisible(true) },
                shape = CircleShape,
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("New group") },
            )
        },
    ) { padding ->
        if (state.isLoading && state.groups.isEmpty()) {
            Box(Modifier.padding(padding).fillMaxSize()) {
                LoadingScreen(message = "Loading your groups…")
            }
        } else {
            LazyColumn(
                modifier = Modifier
                    .padding(padding)
                    .fillMaxSize(),
                contentPadding = PaddingValues(bottom = 112.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                item(key = "header") {
                    GroupsHeader(
                        username = username,
                        isLoggingOut = state.isLoggingOut,
                        onJoin = { viewModel.setAcceptDialogVisible(true) },
                        onRefresh = viewModel::load,
                        onLogout = viewModel::logout,
                    )
                }

                item(key = "hero") {
                    GroupsHero(
                        groupCount = state.groups.size,
                        memberCount = totalMembers,
                        modifier = Modifier.padding(horizontal = 20.dp),
                    )
                }

                if (state.groups.isEmpty()) {
                    item(key = "empty") {
                        EmptyState(
                            icon = Icons.Default.Person,
                            title = "No groups yet",
                            message = "Create your first group and invite the people you share expenses with.",
                            actionLabel = "Create a group",
                            onAction = { viewModel.setCreateDialogVisible(true) },
                            modifier = Modifier.padding(top = 16.dp),
                        )
                    }
                } else {
                    item(key = "section") {
                        SectionHeader(
                            title = "Your groups",
                            subtitle = if (state.groups.size == 1) {
                                "1 group"
                            } else {
                                "${state.groups.size} groups"
                            },
                            modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 6.dp),
                        )
                    }
                    items(state.groups, key = { it.id }) { group ->
                        GroupCard(
                            group = group,
                            onClick = { onOpenGroup(group.id) },
                            modifier = Modifier.padding(horizontal = 20.dp),
                        )
                    }
                }
            }
        }
    }

    if (state.showCreateDialog) {
        CreateGroupDialog(
            isCreating = state.isCreating,
            error = state.createError,
            onDismiss = { if (!state.isCreating) viewModel.setCreateDialogVisible(false) },
            onCreate = { viewModel.createGroup(it) },
        )
    }

    if (state.showAcceptDialog) {
        AcceptInviteDialog(
            isAccepting = state.isAccepting,
            error = state.acceptError,
            onDismiss = { viewModel.setAcceptDialogVisible(false) },
            onAccept = { viewModel.acceptInvite(it) },
        )
    }
}

@Composable
private fun GroupsHeader(
    username: String?,
    isLoggingOut: Boolean,
    onJoin: () -> Unit,
    onRefresh: () -> Unit,
    onLogout: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }
    val greeting = remember {
        when (LocalTime.now().hour) {
            in 0..11 -> "Good morning"
            in 12..17 -> "Good afternoon"
            else -> "Good evening"
        }
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 20.dp, end = 12.dp, top = 20.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BrandMark(size = 44.dp)
        Spacer(Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = greeting,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = username ?: "Splitify",
                style = MaterialTheme.typography.titleLarge,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        HeaderAction(
            icon = Icons.Default.Add,
            contentDescription = "Join with invite",
            enabled = !isLoggingOut,
            onClick = onJoin,
        )
        Spacer(Modifier.width(4.dp))
        if (isLoggingOut) {
            Box(
                modifier = Modifier.size(44.dp),
                contentAlignment = Alignment.Center,
            ) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), strokeWidth = 2.dp)
            }
        } else {
            Box {
                HeaderAction(
                    icon = Icons.Default.MoreVert,
                    contentDescription = "More options",
                    enabled = true,
                    onClick = { menuExpanded = true },
                )
                DropdownMenu(
                    expanded = menuExpanded,
                    onDismissRequest = { menuExpanded = false },
                ) {
                    DropdownMenuItem(
                        text = { Text("Refresh") },
                        leadingIcon = {
                            Icon(Icons.Default.Refresh, contentDescription = null)
                        },
                        onClick = {
                            menuExpanded = false
                            onRefresh()
                        },
                    )
                    DropdownMenuItem(
                        text = { Text("Log out") },
                        leadingIcon = {
                            Icon(
                                Icons.AutoMirrored.Filled.ExitToApp,
                                contentDescription = null,
                            )
                        },
                        onClick = {
                            menuExpanded = false
                            onLogout()
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun HeaderAction(
    icon: ImageVector,
    contentDescription: String,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Surface(
        onClick = onClick,
        enabled = enabled,
        shape = CircleShape,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.size(44.dp),
    ) {
        Box(contentAlignment = Alignment.Center) {
            Icon(
                imageVector = icon,
                contentDescription = contentDescription,
                modifier = Modifier.size(21.dp),
            )
        }
    }
}

@Composable
private fun GroupsHero(
    groupCount: Int,
    memberCount: Long,
    modifier: Modifier = Modifier,
) {
    val gradient = MaterialTheme.splitify.heroGradient
    Box(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(26.dp))
            .background(Brush.linearGradient(gradient)),
    ) {
        Box(
            modifier = Modifier
                .offset(x = 250.dp, y = (-40).dp)
                .size(160.dp)
                .background(Color.White.copy(alpha = 0.09f), CircleShape),
        )
        Box(
            modifier = Modifier
                .offset(x = (-40).dp, y = 90.dp)
                .size(120.dp)
                .background(Color.White.copy(alpha = 0.06f), CircleShape),
        )
        Column(modifier = Modifier.padding(20.dp)) {
            Text(
                text = "YOUR SPACE",
                style = MaterialTheme.typography.labelSmall,
                letterSpacing = 1.8.sp,
                color = Color.White.copy(alpha = 0.72f),
            )
            Spacer(Modifier.height(14.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                HeroStat(value = groupCount.toString(), label = "groups")
                Spacer(Modifier.width(20.dp))
                Box(
                    modifier = Modifier
                        .width(1.dp)
                        .height(38.dp)
                        .background(Color.White.copy(alpha = 0.22f)),
                )
                Spacer(Modifier.width(20.dp))
                HeroStat(value = memberCount.toString(), label = "people")
            }
            Spacer(Modifier.height(14.dp))
            Text(
                text = "Everything split fairly, all in one place.",
                style = MaterialTheme.typography.bodySmall,
                color = Color.White.copy(alpha = 0.75f),
            )
        }
    }
}

@Composable
private fun HeroStat(value: String, label: String) {
    Column {
        Text(
            text = value,
            style = MaterialTheme.typography.headlineLarge,
            color = Color.White,
        )
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = Color.White.copy(alpha = 0.72f),
        )
    }
}

@Composable
private fun GroupCard(
    group: GroupSummaryDto,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    SplitifyCard(onClick = onClick, modifier = modifier) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Avatar(name = group.name, seed = group.id, size = 52.dp)
            Spacer(Modifier.width(14.dp))
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = group.name,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false),
                    )
                    if (group.isAdmin) {
                        Spacer(Modifier.width(8.dp))
                        Pill(
                            text = "Admin",
                            containerColor = MaterialTheme.colorScheme.primaryContainer,
                            contentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                    }
                }
                Spacer(Modifier.height(3.dp))
                Text(
                    text = if (group.memberCount == 1L) {
                        "1 member"
                    } else {
                        "${group.memberCount} members"
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                relativeDate(group.updatedAt)?.let { updated ->
                    Text(
                        text = when (updated) {
                            "Today" -> "Active today"
                            "Yesterday" -> "Active yesterday"
                            else -> "Active $updated"
                        },
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.75f),
                    )
                }
            }
            Icon(
                imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                modifier = Modifier.size(22.dp),
            )
        }
    }
}

@Composable
private fun AcceptInviteDialog(
    isAccepting: Boolean,
    error: String?,
    onDismiss: () -> Unit,
    onAccept: (String) -> Unit,
) {
    var inviteInput by rememberSaveable { mutableStateOf("") }

    AlertDialog(
        onDismissRequest = onDismiss,
        shape = RoundedCornerShape(28.dp),
        title = { Text("Join a group") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(
                    text = "Paste the invite link or code you received.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                OutlinedTextField(
                    value = inviteInput,
                    onValueChange = { inviteInput = it },
                    label = { Text("Invite link or code") },
                    singleLine = true,
                    enabled = !isAccepting,
                    isError = error != null,
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
                onClick = { onAccept(inviteInput) },
                enabled = !isAccepting && inviteInput.isNotBlank(),
            ) {
                if (isAccepting) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        strokeWidth = 2.dp,
                    )
                } else {
                    Text("Join")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isAccepting) {
                Text("Cancel")
            }
        },
    )
}

@Composable
private fun CreateGroupDialog(
    isCreating: Boolean,
    error: String?,
    onDismiss: () -> Unit,
    onCreate: (String) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf("") }

    AlertDialog(
        onDismissRequest = onDismiss,
        shape = RoundedCornerShape(28.dp),
        title = { Text("New group") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(
                    text = "Give your group a name — you can invite people right after.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Group name") },
                    singleLine = true,
                    enabled = !isCreating,
                    isError = error != null,
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
                onClick = { onCreate(name) },
                enabled = !isCreating && name.isNotBlank(),
            ) {
                if (isCreating) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        strokeWidth = 2.dp,
                    )
                } else {
                    Text("Create", fontWeight = FontWeight.SemiBold)
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isCreating) {
                Text("Cancel")
            }
        },
    )
}
