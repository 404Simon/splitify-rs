package com.splitify.app.ui.groups.tabs

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.IconBubble
import com.splitify.app.ui.components.Pill
import com.splitify.app.ui.components.SplitifyCard
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
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
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
import com.splitify.app.data.dto.InviteListItemDto
import com.splitify.app.data.invites.InviteRepository
import com.splitify.app.data.network.userMessage
import com.splitify.app.ui.components.LoadingScreen
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class InvitesUiState(
    val invites: List<InviteListItemDto> = emptyList(),
    val isLoading: Boolean = true,
    val isCreator: Boolean = true,
    val showCreateDialog: Boolean = false,
    val isCreating: Boolean = false,
    val createError: String? = null,
    val deleteTarget: InviteListItemDto? = null,
    val isDeleting: Boolean = false,
    val deleteError: String? = null,
) {
    /**
     * The list is populated from the cached group snapshot, so members see any
     * invites already cached. Only an empty list on a member's device falls back
     * to the creator-only message (the server returns 403 for members).
     */
    val creatorOnly: Boolean get() = !isCreator && invites.isEmpty()
}

class InvitesViewModel(
    private val groupId: Long,
    private val inviteRepository: InviteRepository,
    private val authRepository: AuthRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(InvitesUiState())
    val uiState: StateFlow<InvitesUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            combine(
                inviteRepository.observeInvites(groupId),
                inviteRepository.observeCreatorId(groupId),
                authRepository.user,
            ) { invites, creatorId, user ->
                Triple(invites, creatorId, user?.id)
            }.collect { (invites, creatorId, currentUserId) ->
                _uiState.update {
                    it.copy(
                        invites = invites,
                        isLoading = false,
                        isCreator = creatorId == null ||
                            currentUserId == null ||
                            creatorId == currentUserId,
                    )
                }
            }
        }
    }

    fun openCreate() {
        _uiState.update { it.copy(showCreateDialog = true, createError = null) }
    }

    fun dismissCreate() {
        if (_uiState.value.isCreating) return
        _uiState.update { it.copy(showCreateDialog = false, createError = null) }
    }

    fun createInvite(name: String, isReusable: Boolean, durationDaysText: String) {
        val trimmedName = name.trim()
        val days = durationDaysText.trim().toIntOrNull()
        val validationError = when {
            trimmedName.length > 128 -> "Names are limited to 128 characters."
            days == null || days !in 1..30 -> "Duration must be between 1 and 30 days."
            else -> null
        }
        if (validationError != null) {
            _uiState.update { it.copy(createError = validationError) }
            return
        }
        if (_uiState.value.isCreating) return
        val validDays = days ?: return
        _uiState.update { it.copy(isCreating = true, createError = null) }

        viewModelScope.launch {
            inviteRepository.createInvite(
                groupId = groupId,
                name = trimmedName.ifEmpty { null },
                isReusable = isReusable,
                durationDays = validDays.toLong(),
            )
                .onSuccess {
                    // Online-only call: the sync engine refreshes the cached snapshot.
                    _uiState.update {
                        it.copy(
                            isCreating = false,
                            showCreateDialog = false,
                            createError = null,
                        )
                    }
                }
                .onFailure { error ->
                    _uiState.update {
                        it.copy(isCreating = false, createError = error.userMessage())
                    }
                }
        }
    }

    fun requestDelete(invite: InviteListItemDto) {
        _uiState.update { it.copy(deleteTarget = invite, deleteError = null) }
    }

    fun dismissDelete() {
        if (_uiState.value.isDeleting) return
        _uiState.update { it.copy(deleteTarget = null, deleteError = null) }
    }

    fun confirmDelete() {
        val target = _uiState.value.deleteTarget ?: return
        if (_uiState.value.isDeleting) return
        _uiState.update { it.copy(isDeleting = true, deleteError = null) }
        viewModelScope.launch {
            inviteRepository.deleteInvite(groupId, target.uuid)
                .onSuccess {
                    _uiState.update {
                        it.copy(isDeleting = false, deleteTarget = null, deleteError = null)
                    }
                }
                .onFailure { error ->
                    _uiState.update {
                        it.copy(isDeleting = false, deleteError = error.userMessage())
                    }
                }
        }
    }

    companion object {
        fun factory(
            groupId: Long,
            inviteRepository: InviteRepository,
            authRepository: AuthRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { InvitesViewModel(groupId, inviteRepository, authRepository) }
        }
    }
}

@Composable
fun InvitesTab(
    groupId: Long,
    inviteRepository: InviteRepository,
    authRepository: AuthRepository,
) {
    val viewModel: InvitesViewModel = viewModel(
        key = "invites-$groupId",
        factory = InvitesViewModel.factory(groupId, inviteRepository, authRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val context = LocalContext.current

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.invites.isEmpty() ->
                LoadingScreen(message = "Loading invites…")

            state.creatorOnly -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                EmptyState(
                    icon = Icons.Default.Lock,
                    title = "Creator only",
                    message = "Only the group creator can create and revoke invite links.",
                )
            }

            state.invites.isEmpty() -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                EmptyState(
                    icon = Icons.Default.Share,
                    title = "No invite links yet",
                    message = "Create a link and share it with anyone you want in this group.",
                    actionLabel = "Create invite",
                    onAction = { viewModel.openCreate() },
                )
            }

            else -> LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(
                    start = 20.dp,
                    end = 20.dp,
                    top = 4.dp,
                    bottom = 96.dp,
                ),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                items(state.invites, key = { it.uuid }) { invite ->
                    InviteCard(
                        invite = invite,
                        link = inviteRepository.inviteLink(invite.uuid),
                        onCopy = { link -> copyInviteLink(context, link) },
                        onShare = { link -> shareInviteLink(context, link) },
                        onRevoke = { viewModel.requestDelete(invite) },
                    )
                }
            }
        }

        if (!state.creatorOnly && (!state.isLoading || state.invites.isNotEmpty())) {
            ExtendedFloatingActionButton(
                onClick = { viewModel.openCreate() },
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(16.dp),
                shape = CircleShape,
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("New invite") },
            )
        }
    }

    if (state.showCreateDialog) {
        CreateInviteDialog(
            isCreating = state.isCreating,
            error = state.createError,
            onDismiss = { viewModel.dismissCreate() },
            onCreate = { name, reusable, days ->
                viewModel.createInvite(name, reusable, days)
            },
        )
    }

    state.deleteTarget?.let { invite ->
        AlertDialog(
            onDismissRequest = { viewModel.dismissDelete() },
            shape = RoundedCornerShape(28.dp),
            title = { Text("Revoke invite") },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        "Revoke \"${invite.name ?: "this invite"}\"? " +
                            "The link will stop working immediately.",
                    )
                    state.deleteError?.let { message ->
                        Text(
                            text = message,
                            color = MaterialTheme.colorScheme.error,
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
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
                        Text("Revoke", color = MaterialTheme.colorScheme.error)
                    }
                }
            },
            dismissButton = {
                TextButton(
                    onClick = { viewModel.dismissDelete() },
                    enabled = !state.isDeleting,
                ) {
                    Text("Cancel")
                }
            },
        )
    }
}

@Composable
private fun InviteCard(
    invite: InviteListItemDto,
    link: String?,
    onCopy: (String) -> Unit,
    onShare: (String) -> Unit,
    onRevoke: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }

    SplitifyCard(contentPadding = PaddingValues(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            IconBubble(
                icon = Icons.Default.Share,
                size = 44.dp,
                containerColor = MaterialTheme.colorScheme.tertiaryContainer,
                contentColor = MaterialTheme.colorScheme.onTertiaryContainer,
            )
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = invite.name ?: "Untitled invite",
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false),
                    )
                    if (invite.isReusable) {
                        Spacer(Modifier.width(8.dp))
                        Pill(
                            text = "Reusable",
                            containerColor = MaterialTheme.colorScheme.tertiaryContainer,
                            contentColor = MaterialTheme.colorScheme.onTertiaryContainer,
                        )
                    }
                }
                Text(
                    text = "Expires ${invite.expirationDate}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Box {
                IconButton(onClick = { menuExpanded = true }) {
                    Icon(
                        imageVector = Icons.Default.MoreVert,
                        contentDescription = "Invite options",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                DropdownMenu(
                    expanded = menuExpanded,
                    onDismissRequest = { menuExpanded = false },
                ) {
                    DropdownMenuItem(
                        text = { Text("Revoke invite") },
                        leadingIcon = {
                            Icon(
                                imageVector = Icons.Default.Delete,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.error,
                            )
                        },
                        onClick = {
                            menuExpanded = false
                            onRevoke()
                        },
                    )
                }
            }
        }

        Spacer(Modifier.height(12.dp))

        if (link == null) {
            Text(
                text = "No server address is configured, so this link cannot be built.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
            )
        } else {
            Surface(
                shape = RoundedCornerShape(14.dp),
                color = MaterialTheme.colorScheme.surfaceContainerLow,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        imageVector = Icons.Default.Lock,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(15.dp),
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = link,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f),
                    )
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                TextButton(onClick = { onCopy(link) }) {
                    Text("Copy link")
                }
                TextButton(onClick = { onShare(link) }) {
                    Text("Share")
                }
            }
        }
    }
}

@Composable
private fun CreateInviteDialog(
    isCreating: Boolean,
    error: String?,
    onDismiss: () -> Unit,
    onCreate: (String, Boolean, String) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf("") }
    var reusable by rememberSaveable { mutableStateOf(false) }
    var durationDays by rememberSaveable { mutableStateOf("7") }

    val days = durationDays.trim().toIntOrNull()
    val valid = days != null && days in 1..30

    AlertDialog(
        onDismissRequest = onDismiss,
        shape = RoundedCornerShape(28.dp),
        title = { Text("New invite") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Name (optional)") },
                    singleLine = true,
                    enabled = !isCreating,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )
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
                            Text(
                                text = "Reusable",
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            Text(
                                text = "The link can be used more than once.",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        Spacer(Modifier.width(12.dp))
                        Switch(
                            checked = reusable,
                            onCheckedChange = { reusable = it },
                            enabled = !isCreating,
                        )
                    }
                }
                OutlinedTextField(
                    value = durationDays,
                    onValueChange = { durationDays = it },
                    label = { Text("Valid for (1–30 days)") },
                    singleLine = true,
                    enabled = !isCreating,
                    isError = durationDays.isNotBlank() && !valid,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
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
                onClick = { onCreate(name, reusable, durationDays) },
                enabled = !isCreating && valid,
            ) {
                if (isCreating) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        strokeWidth = 2.dp,
                    )
                } else {
                    Text("Create")
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

private fun copyInviteLink(context: Context, link: String) {
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    clipboard.setPrimaryClip(ClipData.newPlainText("Splitify invite", link))
    Toast.makeText(context, "Invite link copied", Toast.LENGTH_SHORT).show()
}

private fun shareInviteLink(context: Context, link: String) {
    val intent = Intent(Intent.ACTION_SEND).apply {
        type = "text/plain"
        putExtra(Intent.EXTRA_TEXT, link)
    }
    context.startActivity(Intent.createChooser(intent, "Share invite"))
}
