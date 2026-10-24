package com.splitify.app.data.auth

import com.splitify.app.data.dto.AuthResponseDto
import com.splitify.app.data.dto.LoginRequest
import com.splitify.app.data.dto.RegisterRequest
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.config.ServerConfig
import com.splitify.app.data.dto.UserDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow

class AuthRepository(
    private val api: ApiClient,
    private val session: SessionManager,
    private val localStore: LocalStore,
    private val serverConfig: ServerConfig,
) {

    val token: Flow<String?> = session.token

    val user: Flow<UserDto?> = session.user

    suspend fun login(serverUrl: String, username: String, password: String): Result<UserDto> =
        apiResult {
            useServer(serverUrl)
            val response = api.post(
                path = "auth/login",
                body = LoginRequest(username = username, password = password),
                bodySerializer = LoginRequest.serializer(),
                serializer = AuthResponseDto.serializer(),
                authenticated = false,
            )
            session.save(response.token, response.user)
            localStore.onUserChanged(response.user.id)
            response.user
        }

    suspend fun register(
        serverUrl: String,
        username: String,
        password: String,
        email: String?,
    ): Result<UserDto> = apiResult {
        useServer(serverUrl)
        val response = api.post(
            path = "auth/register",
            body = RegisterRequest(username = username, password = password, email = email),
            bodySerializer = RegisterRequest.serializer(),
            serializer = AuthResponseDto.serializer(),
            authenticated = false,
        )
        session.save(response.token, response.user)
        localStore.onUserChanged(response.user.id)
        response.user
    }

    /**
     * Points the client at [serverUrl] before the first request.
     *
     * A token and the cached groups belong to the server that issued them, so
     * moving to a different server must drop both instead of leaking one
     * deployment's data into the other.
     */
    private suspend fun useServer(serverUrl: String) {
        val previous = serverConfig.current()
        if (previous != null && previous != serverUrl) {
            session.clear()
            localStore.wipe()
        }
        serverConfig.save(serverUrl)
    }

    suspend fun me(): Result<UserDto> = apiResult {
        val user = api.get("auth/me", UserDto.serializer())
        session.updateUser(user)
        localStore.onUserChanged(user.id)
        user
    }

    /**
     * Clears the local session immediately so logout works offline, then tries
     * to revoke the token on the server. A failed revoke is harmless: the token
     * expires on the server anyway.
     */
    suspend fun logout() {
        val token = session.token.value
        session.clear()
        if (token != null) {
            try {
                api.postNoContentWithToken("auth/logout", token)
            } catch (error: CancellationException) {
                throw error
            } catch (_: Exception) {
                // Local session is already gone.
            }
        }
    }

}
