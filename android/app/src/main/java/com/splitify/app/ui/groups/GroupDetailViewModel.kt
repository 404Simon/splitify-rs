package com.splitify.app.ui.groups

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.dto.GroupDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.groups.GroupRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class GroupDetailUiState(
    val group: GroupDto? = null,
    val members: List<GroupMemberDto> = emptyList(),
    val currentUserId: Long? = null,
    val isLoading: Boolean = true,
    val error: String? = null,
)

/**
 * Group detail is rendered entirely from the local snapshot cache. The current
 * user comes from [SessionManager], so no `/auth/me` round trip is needed.
 */
class GroupDetailViewModel(
    private val groupId: Long,
    private val groupRepository: GroupRepository,
    session: SessionManager,
) : ViewModel() {

    private val _uiState = MutableStateFlow(GroupDetailUiState())
    val uiState: StateFlow<GroupDetailUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            session.user.collect { user ->
                _uiState.update { it.copy(currentUserId = user?.id) }
            }
        }
        viewModelScope.launch {
            kotlinx.coroutines.flow.combine(
                groupRepository.observeGroup(groupId),
                groupRepository.isLoaded,
            ) { group, loaded ->
                group to loaded
            }.collect { (group, loaded) ->
                _uiState.update { state ->
                    when {
                        group != null -> state.copy(group = group, isLoading = false, error = null)

                        !loaded -> state.copy(isLoading = true)

                        state.group != null -> state.copy(
                            group = null,
                            isLoading = false,
                            error = "This group is no longer available.",
                        )

                        else -> state.copy(
                            isLoading = false,
                            error = "This group isn't available offline yet.",
                        )
                    }
                }
            }
        }
        viewModelScope.launch {
            groupRepository.observeMembers(groupId).collect { members ->
                _uiState.update { it.copy(members = members) }
            }
        }
        load()
    }

    /** Requests a background refresh; cached data stays visible. */
    fun load() {
        groupRepository.refresh()
    }

    companion object {
        fun factory(
            groupId: Long,
            groupRepository: GroupRepository,
            session: SessionManager,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { GroupDetailViewModel(groupId, groupRepository, session) }
        }
    }
}
