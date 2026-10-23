package com.splitify.app.ui.groups.tabs

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.widget.Toast
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
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import com.splitify.app.data.network.ApiException
import com.splitify.app.data.network.userMessage
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class InvitesUiState(
    val invites: List<InviteListItemDto> = emptyList(),
    val isLoading: Boolean = true,
    val error: String? = null,
    val forbidden: Boolean = false,
    val showCreateDialog: Boolean = false,
    val isCreating: Boolean = false,
    val createError: String? = null,
    val deleteTarget: InviteListItemDto? = null,
    val isDeleting: Boolean = false,
)

class InvitesViewModel(
    private val groupId: Long,
    private val inviteRepository: InviteRepository,
    private val authRepository: AuthRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(InvitesUiState())
    val uiState: StateFlow<InvitesUiState> = _uiState.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null, forbidden = false) }
            inviteRepository.listGroupInvites(groupId)
                .onSuccess { invites ->
                    _uiState.update {
                        it.copy(invites = invites, isLoading = false, forbidden = false)
                    }
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    if (error is ApiException.Http && error.statusCode == 403) {
                        _uiState.update {
                            it.copy(
                                invites = emptyList(),
                                isLoading = false,
                                error = null,
                                forbidden = true,
                            )
                        }
                    } else {
                        _uiState.update {
                            it.copy(isLoading = false, error = error.userMessage())
                        }
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
                    _uiState.update {
                        it.copy(
                            isCreating = false,
                            showCreateDialog = false,
                            createError = null,
                        )
                    }
                    load()
                }
                .onFailure { error ->
                    handleUnauthorized(error)
                    _uiState.update {
                        it.copy(isCreating = false, createError = error.userMessage())
                    }
                }
        }
    }

    fun requestDelete(invite: InviteListItemDto) {
        _uiState.update { it.copy(deleteTarget = invite) }
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
            inviteRepository.deleteInvite(groupId, target.uuid)
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

    LaunchedEffect(Unit) { viewModel.load() }

    Box(modifier = Modifier.fillMaxSize()) {
        when {
            state.isLoading && state.invites.isEmpty() ->
                LoadingScreen(message = "Loading invites…")

            state.forbidden -> CenteredInviteMessage(
                title = "Creator only",
                message = "Only the group creator can manage invites.",
                icon = { Icon(Icons.Default.Lock, contentDescription = null) },
            )

            state.error != null && state.invites.isEmpty() -> CenteredInviteError(
                message = state.error.orEmpty(),
                onRetry = { viewModel.load() },
            )

            state.invites.isEmpty() -> EmptyInvites(onCreate = { viewModel.openCreate() })

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
        }

        if (!state.forbidden && (!state.isLoading || state.invites.isNotEmpty())) {
            ExtendedFloatingActionButton(
                onClick = { viewModel.openCreate() },
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(16.dp),
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
            title = { Text("Revoke invite") },
            text = {
                Text(
                    "Revoke \"${invite.name ?: "this invite"}\"? " +
                        "The link will stop working immediately.",
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
    link: String,
    onCopy: (String) -> Unit,
    onShare: (String) -> Unit,
    onRevoke: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = invite.name ?: "Untitled invite",
                    modifier = Modifier.weight(1f),
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (invite.isReusable) {
                    Surface(
                        shape = MaterialTheme.shapes.small,
                        color = MaterialTheme.colorScheme.secondaryContainer,
                        contentColor = MaterialTheme.colorScheme.onSecondaryContainer,
                    ) {
                        Text(
                            text = "Reusable",
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                            style = MaterialTheme.typography.labelSmall,
                        )
                    }
                }
            }

            Text(
                text = "Expires ${invite.expirationDate}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            Text(
                text = link,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )

            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                TextButton(onClick = { onCopy(link) }) {
                    Text("Copy link")
                }
                TextButton(onClick = { onShare(link) }) {
                    Text("Share")
                }
                TextButton(onClick = onRevoke) {
                    Text("Revoke", color = MaterialTheme.colorScheme.error)
                }
            }
        }
    }
}

@Composable
private fun EmptyInvites(onCreate: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Default.Lock,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(40.dp),
        )
        Spacer(Modifier.height(12.dp))
        Text(text = "No invites yet", style = MaterialTheme.typography.titleMedium)
        Text(
            text = "Create an invite link and share it with someone you want to join.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = 4.dp),
        )
        Button(onClick = onCreate, modifier = Modifier.padding(top = 16.dp)) {
            Text("Create invite")
        }
    }
}

@Composable
private fun CenteredInviteMessage(
    title: String,
    message: String,
    icon: @Composable () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Box(
            modifier = Modifier.size(40.dp),
            contentAlignment = Alignment.Center,
        ) {
            icon()
        }
        Spacer(Modifier.height(12.dp))
        Text(text = title, style = MaterialTheme.typography.titleMedium)
        Spacer(Modifier.height(4.dp))
        Text(
            text = message,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
private fun CenteredInviteError(
    message: String,
    onRetry: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Default.Warning,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.error,
            modifier = Modifier.align(Alignment.CenterHorizontally),
        )
        Spacer(Modifier.height(12.dp))
        ErrorBanner(message = message, onRetry = onRetry)
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
        title = { Text("New invite") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Name (optional)") },
                    singleLine = true,
                    enabled = !isCreating,
                    modifier = Modifier.fillMaxWidth(),
                )
                Row(
                    modifier = Modifier.fillMaxWidth(),
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
                OutlinedTextField(
                    value = durationDays,
                    onValueChange = { durationDays = it },
                    label = { Text("Duration (1–30 days)") },
                    singleLine = true,
                    enabled = !isCreating,
                    isError = durationDays.isNotBlank() && !valid,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
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
