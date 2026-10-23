package com.splitify.app.ui.groups

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.dto.GroupSummaryDto
import com.splitify.app.data.groups.GroupRepository
import com.splitify.app.data.invites.InviteRepository
import com.splitify.app.data.network.userMessage
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GroupsUiState(
    val groups: List<GroupSummaryDto> = emptyList(),
    val isLoading: Boolean = true,
    val isCreating: Boolean = false,
    val createError: String? = null,
    val showCreateDialog: Boolean = false,
    val isLoggingOut: Boolean = false,
    val showAcceptDialog: Boolean = false,
    val isAccepting: Boolean = false,
    val acceptError: String? = null,
    val acceptedGroupId: Long? = null,
)

/**
 * Reads the cached group list from [GroupRepository] and never blocks on the
 * network. Sync state is surfaced globally by the SyncBanner, so this screen
 * only owns local validation errors and dialog state.
 */
class GroupsViewModel(
    private val groupRepository: GroupRepository,
    private val authRepository: AuthRepository,
    private val inviteRepository: InviteRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(GroupsUiState())
    val uiState: StateFlow<GroupsUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            kotlinx.coroutines.flow.combine(
                groupRepository.observeGroups(),
                groupRepository.isLoaded,
            ) { groups, loaded ->
                groups to loaded
            }.collect { (groups, loaded) ->
                _uiState.update {
                    it.copy(groups = groups, isLoading = !loaded)
                }
            }
        }
        load()
    }

    /** Requests a background refresh; cached data stays visible throughout. */
    fun load() {
        groupRepository.refresh()
    }

    fun setCreateDialogVisible(visible: Boolean) {
        _uiState.update { it.copy(showCreateDialog = visible, createError = null) }
    }

    fun createGroup(name: String) {
        val trimmed = name.trim()
        if (trimmed.isEmpty()) {
            _uiState.update { it.copy(createError = "Enter a group name.") }
            return
        }
        if (_uiState.value.isCreating) return
        _uiState.update { it.copy(isCreating = true, createError = null) }
        viewModelScope.launch {
            groupRepository.createGroup(trimmed)
                .onSuccess {
                    // The observed flow adds the new group; no reload needed.
                    _uiState.update {
                        it.copy(isCreating = false, showCreateDialog = false, createError = null)
                    }
                }
                .onFailure {
                    _uiState.update {
                        it.copy(isCreating = false, createError = "Could not create the group.")
                    }
                }
        }
    }

    fun setAcceptDialogVisible(visible: Boolean) {
        if (_uiState.value.isAccepting) return
        _uiState.update { it.copy(showAcceptDialog = visible, acceptError = null) }
    }

    fun acceptInvite(input: String) {
        if (_uiState.value.isAccepting) return
        val uuid = inviteRepository.extractInviteUuid(input)
        if (uuid == null) {
            _uiState.update { it.copy(acceptError = "Enter a valid invite link or code.") }
            return
        }
        _uiState.update { it.copy(isAccepting = true, acceptError = null) }
        viewModelScope.launch {
            inviteRepository.acceptInvite(uuid)
                .onSuccess { groupId ->
                    _uiState.update {
                        it.copy(
                            isAccepting = false,
                            showAcceptDialog = false,
                            acceptError = null,
                            acceptedGroupId = groupId,
                        )
                    }
                    groupRepository.refresh()
                }
                .onFailure { error ->
                    _uiState.update {
                        it.copy(isAccepting = false, acceptError = error.userMessage())
                    }
                }
        }
    }

    fun consumeAcceptedGroup() {
        _uiState.update { it.copy(acceptedGroupId = null) }
    }

    fun logout() {
        if (_uiState.value.isLoggingOut) return
        _uiState.update { it.copy(isLoggingOut = true) }
        viewModelScope.launch {
            authRepository.logout()
            _uiState.update { it.copy(isLoggingOut = false) }
        }
    }

    companion object {
        fun factory(
            groupRepository: GroupRepository,
            authRepository: AuthRepository,
            inviteRepository: InviteRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { GroupsViewModel(groupRepository, authRepository, inviteRepository) }
        }
    }
}
