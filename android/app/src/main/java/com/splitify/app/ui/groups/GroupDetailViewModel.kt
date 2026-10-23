package com.splitify.app.ui.groups

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.dto.GroupDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.groups.GroupRepository
import com.splitify.app.data.network.ApiException
import com.splitify.app.data.network.userMessage
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
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

class GroupDetailViewModel(
    private val groupId: Long,
    private val groupRepository: GroupRepository,
    private val authRepository: AuthRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(GroupDetailUiState())
    val uiState: StateFlow<GroupDetailUiState> = _uiState.asStateFlow()

    init {
        load()
    }

    fun load() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            coroutineScope {
                val groupDeferred = async { groupRepository.getGroup(groupId) }
                val membersDeferred = async { groupRepository.getMembers(groupId) }
                val userDeferred = async { authRepository.me() }
                val groupResult = groupDeferred.await()
                val membersResult = membersDeferred.await()
                val userResult = userDeferred.await()
                val failure = groupResult.exceptionOrNull()
                    ?: membersResult.exceptionOrNull()
                    ?: userResult.exceptionOrNull()

                if (failure != null) {
                    handleUnauthorized(failure)
                    _uiState.update { it.copy(isLoading = false, error = failure.userMessage()) }
                } else {
                    _uiState.update {
                        it.copy(
                            group = groupResult.getOrNull(),
                            members = membersResult.getOrDefault(emptyList()),
                            currentUserId = userResult.getOrNull()?.id,
                            isLoading = false,
                        )
                    }
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
            groupRepository: GroupRepository,
            authRepository: AuthRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { GroupDetailViewModel(groupId, groupRepository, authRepository) }
        }
    }
}
