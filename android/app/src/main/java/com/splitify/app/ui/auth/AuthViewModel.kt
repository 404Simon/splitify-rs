package com.splitify.app.ui.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.dto.UserDto
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

class AuthViewModel(
    private val authRepository: AuthRepository,
    private val sessionManager: SessionManager,
) : ViewModel() {

    private val _uiState = MutableStateFlow(AuthUiState())
    val uiState: StateFlow<AuthUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            sessionManager.awaitReady()
            restoreSession()
        }
        // A token that disappears (logout, or a 401 handled by ApiClient) always
        // ends the session, even if a background refresh is still in flight.
        viewModelScope.launch {
            sessionManager.awaitReady()
            sessionManager.token.collect { token ->
                if (token == null) setSession(SessionState.Unauthenticated)
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

    /**
     * Offline-first restore: a cached token plus cached profile is enough to render
     * the app. The profile is then refreshed in the background, and only a real
     * `401` (handled centrally by `ApiClient.onUnauthorized`) ends the session.
     */
    private suspend fun restoreSession() {
        if (sessionManager.token.value == null) {
            setSession(SessionState.Unauthenticated)
            return
        }

        val cachedUser = sessionManager.user.value
        if (cachedUser == null) {
            // Without a cached profile there is nothing to render, so this path
            // has to wait for the server and fails closed.
            setSession(
                authRepository.me().fold(
                    onSuccess = { SessionState.Authenticated(it) },
                    onFailure = { SessionState.Unauthenticated },
                ),
            )
            return
        }

        setSession(SessionState.Authenticated(cachedUser))
        validateSession()
    }

    private suspend fun validateSession() {
        authRepository.me()
            .onSuccess { user ->
                if (sessionManager.token.value != null) {
                    setSession(SessionState.Authenticated(user))
                }
            }
            // Network failures keep the cached session, otherwise an offline launch
            // would sign the user out. A 401 already cleared the token (and this
            // ViewModel observes that through the token flow).
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

    private fun setSession(session: SessionState) {
        _uiState.update { it.copy(session = session) }
    }

    private fun setError(message: String) {
        _uiState.update { it.copy(error = message) }
    }

    companion object {
        private val USERNAME_PATTERN = Regex("^[\\p{L}\\p{N}_-]{3,50}$")

        fun factory(
            authRepository: AuthRepository,
            sessionManager: SessionManager,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { AuthViewModel(authRepository, sessionManager) }
        }
    }
}
