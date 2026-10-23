package com.splitify.app.ui.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.dto.UserDto
import com.splitify.app.data.network.ApiException
import com.splitify.app.data.network.userMessage
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

sealed interface SessionState {
    data object Checking : SessionState

    data object Unauthenticated : SessionState

    data class Authenticated(val user: UserDto) : SessionState
}

data class AuthUiState(
    val session: SessionState = SessionState.Checking,
    val isSubmitting: Boolean = false,
    val error: String? = null,
)

class AuthViewModel(private val authRepository: AuthRepository) : ViewModel() {

    private val _uiState = MutableStateFlow(AuthUiState())
    val uiState: StateFlow<AuthUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            authRepository.token.collect { token ->
                val current = _uiState.value
                when {
                    token == null ->
                        _uiState.update { it.copy(session = SessionState.Unauthenticated) }

                    current.isSubmitting || current.session is SessionState.Authenticated -> Unit

                    else -> checkSession()
                }
            }
        }
    }

    fun login(username: String, password: String) {
        val name = username.trim()
        if (name.isBlank() || password.isBlank()) {
            setError("Enter your username and password.")
            return
        }
        submit { authRepository.login(name, password) }
    }

    fun register(username: String, email: String, password: String, confirmPassword: String) {
        val name = username.trim()
        when {
            !USERNAME_PATTERN.matches(name) ->
                setError("Username must be 3-50 characters: letters, numbers, _ or -.")
            password.length < 8 -> setError("Password must be at least 8 characters.")
            password != confirmPassword -> setError("Passwords do not match.")
            else -> submit {
                authRepository.register(
                    username = name,
                    password = password,
                    email = email.trim().ifBlank { null },
                )
            }
        }
    }

    fun clearError() {
        _uiState.update { it.copy(error = null) }
    }

    private suspend fun checkSession() {
        _uiState.update { it.copy(session = SessionState.Checking) }
        authRepository.me()
            .onSuccess { user ->
                _uiState.update { it.copy(session = SessionState.Authenticated(user)) }
            }
            .onFailure { error ->
                if (error is ApiException.Http && error.isUnauthorized) {
                    authRepository.clearSession()
                }
                _uiState.update {
                    it.copy(session = SessionState.Unauthenticated, error = error.userMessage())
                }
            }
    }

    private fun submit(action: suspend () -> Result<UserDto>) {
        if (_uiState.value.isSubmitting) return
        _uiState.update { it.copy(isSubmitting = true, error = null) }
        viewModelScope.launch {
            action()
                .onSuccess { user ->
                    _uiState.update {
                        it.copy(
                            session = SessionState.Authenticated(user),
                            isSubmitting = false,
                            error = null,
                        )
                    }
                }
                .onFailure { error ->
                    _uiState.update {
                        it.copy(isSubmitting = false, error = error.userMessage())
                    }
                }
        }
    }

    private fun setError(message: String) {
        _uiState.update { it.copy(error = message) }
    }

    companion object {
        private val USERNAME_PATTERN = Regex("^[\\p{L}\\p{N}_-]{3,50}$")

        fun factory(authRepository: AuthRepository): ViewModelProvider.Factory = viewModelFactory {
            initializer { AuthViewModel(authRepository) }
        }
    }
}
