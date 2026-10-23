package com.splitify.app.data.auth

import com.splitify.app.data.dto.UserDto
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

/**
 * Single source of truth for "who is signed in".
 *
 * [isReady] becomes true after DataStore delivered its first value, so the app
 * can distinguish "still loading" from "no session" without flashing the login
 * screen on every cold start.
 */
class SessionManager(
    private val tokenStore: TokenStore,
    private val scope: CoroutineScope,
) {

    private val _token = MutableStateFlow<String?>(null)
    val token: StateFlow<String?> = _token.asStateFlow()

    private val _user = MutableStateFlow<UserDto?>(null)
    val user: StateFlow<UserDto?> = _user.asStateFlow()

    private val _isReady = MutableStateFlow(false)
    val isReady: StateFlow<Boolean> = _isReady.asStateFlow()

    init {
        scope.launch {
            // `combine` waits for both flows' first value, so a cold start never
            // briefly looks unauthenticated while the token is still decrypting.
            kotlinx.coroutines.flow.combine(tokenStore.token, tokenStore.user) { token, user ->
                token to user
            }.collect { (token, user) ->
                _token.value = token
                _user.value = user
                _isReady.value = true
            }
        }
    }

    suspend fun awaitReady() {
        isReady.first { it }
    }

    suspend fun requireUser(): UserDto =
        user.value ?: user.first { it != null } ?: error("No signed-in user")

    suspend fun save(token: String, user: UserDto) {
        tokenStore.save(token, user)
        _token.value = token
        _user.value = user
    }

    suspend fun updateUser(user: UserDto) {
        tokenStore.saveUser(user)
        _user.value = user
    }

    suspend fun clear() {
        tokenStore.clear()
        _token.value = null
        _user.value = null
    }

    /** Clears the session from a non-suspending context (e.g. an API callback). */
    fun clearAsync() {
        scope.launch { clear() }
    }
}
