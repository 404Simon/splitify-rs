package com.splitify.app.data.auth

import com.splitify.app.data.dto.AuthResponseDto
import com.splitify.app.data.dto.LoginRequest
import com.splitify.app.data.dto.RegisterRequest
import com.splitify.app.data.dto.UserDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow

class AuthRepository(
    private val api: ApiClient,
    private val tokenStore: TokenStore,
) {

    val token: Flow<String?> = tokenStore.token

    val username: Flow<String?> = tokenStore.username

    suspend fun login(username: String, password: String): Result<UserDto> = apiResult {
        val response = api.post(
            path = "auth/login",
            body = LoginRequest(username = username, password = password),
            bodySerializer = LoginRequest.serializer(),
            serializer = AuthResponseDto.serializer(),
            authenticated = false,
        )
        tokenStore.save(response.token, response.user.username)
        response.user
    }

    suspend fun register(username: String, password: String, email: String?): Result<UserDto> = apiResult {
        val response = api.post(
            path = "auth/register",
            body = RegisterRequest(username = username, password = password, email = email),
            bodySerializer = RegisterRequest.serializer(),
            serializer = AuthResponseDto.serializer(),
            authenticated = false,
        )
        tokenStore.save(response.token, response.user.username)
        response.user
    }

    suspend fun me(): Result<UserDto> = apiResult {
        api.get("auth/me", UserDto.serializer())
    }

    suspend fun logout() {
        try {
            api.postNoContent("auth/logout")
        } catch (error: CancellationException) {
            throw error
        } catch (_: Exception) {
            // Local session is cleared regardless of the server response.
        }
        tokenStore.clear()
    }

    suspend fun clearSession() {
        tokenStore.clear()
    }
}
