package com.splitify.app.data.network

import android.util.Log
import com.splitify.app.BuildConfig
import com.splitify.app.data.dto.ApiErrorDto
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.plugins.logging.LogLevel
import io.ktor.client.plugins.logging.Logger
import io.ktor.client.plugins.logging.Logging
import io.ktor.client.request.header
import io.ktor.client.request.request
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json

class ApiClient(
    baseUrl: String,
    private val tokenProvider: suspend () -> String?,
) {

    private val baseUrl: String = baseUrl.trimEnd('/') + "/"

    init {
        // Fail fast on a release build pointed at plaintext HTTP or the build
        // placeholder instead of silently sending bearer tokens in the clear.
        check(
            BuildConfig.DEBUG ||
                (this.baseUrl.startsWith("https://") && !this.baseUrl.contains(".invalid")),
        ) {
            "Release builds require an HTTPS API base URL (set -PapiBaseUrl=https://...)."
        }
    }

    private val json: Json = Json {
        ignoreUnknownKeys = true
        explicitNulls = false
        encodeDefaults = true
    }

    private val client: HttpClient = HttpClient(OkHttp) {
        expectSuccess = false

        install(ContentNegotiation) {
            json(this@ApiClient.json)
        }

        install(HttpTimeout) {
            requestTimeoutMillis = 20_000
            connectTimeoutMillis = 10_000
            socketTimeoutMillis = 20_000
        }

        install(Logging) {
            logger = object : Logger {
                override fun log(message: String) {
                    if (BuildConfig.DEBUG) Log.d("SplitifyApi", message)
                }
            }
            level = if (BuildConfig.DEBUG) LogLevel.INFO else LogLevel.NONE
            sanitizeHeader { name -> name.equals(HttpHeaders.Authorization, ignoreCase = true) }
        }

        defaultRequest {
            url(this@ApiClient.baseUrl)
        }
    }

    suspend fun <T> get(
        path: String,
        serializer: KSerializer<T>,
        authenticated: Boolean = true,
    ): T = execute(method = HttpMethod.Get, path = path, authenticated = authenticated)
        .decode(serializer)

    suspend fun <B, T> post(
        path: String,
        body: B,
        bodySerializer: KSerializer<B>,
        serializer: KSerializer<T>,
        authenticated: Boolean = true,
    ): T {
        val payload = json.encodeToString(bodySerializer, body)
        return execute(method = HttpMethod.Post, path = path, payload = payload, authenticated = authenticated)
            .decode(serializer)
    }

    suspend fun postNoContent(path: String, authenticated: Boolean = true) {
        execute(method = HttpMethod.Post, path = path, authenticated = authenticated)
    }

    suspend fun <B> patch(
        path: String,
        body: B,
        bodySerializer: KSerializer<B>,
        authenticated: Boolean = true,
    ) {
        val payload = json.encodeToString(bodySerializer, body)
        execute(method = HttpMethod.Patch, path = path, payload = payload, authenticated = authenticated)
    }

    suspend fun delete(path: String, authenticated: Boolean = true) {
        execute(method = HttpMethod.Delete, path = path, authenticated = authenticated)
    }

    private suspend fun execute(
        method: HttpMethod,
        path: String,
        payload: String? = null,
        authenticated: Boolean = true,
    ): HttpResponse {
        val token = if (authenticated) tokenProvider() ?: throw ApiException.Unauthorized() else null

        val response = try {
            client.request(path) {
                this.method = method
                if (payload != null) {
                    contentType(ContentType.Application.Json)
                    setBody(payload)
                }
                if (token != null) {
                    header(HttpHeaders.Authorization, "Bearer $token")
                }
            }
        } catch (error: CancellationException) {
            throw error
        } catch (error: Exception) {
            throw ApiException.Network(error)
        }

        if (!response.status.isSuccess()) {
            throw response.toApiException()
        }
        return response
    }

    private suspend fun <T> HttpResponse.decode(serializer: KSerializer<T>): T {
        val text = try {
            bodyAsText()
        } catch (error: CancellationException) {
            throw error
        } catch (error: Exception) {
            throw ApiException.Parse(error)
        }

        return try {
            json.decodeFromString(serializer, text)
        } catch (error: Exception) {
            throw ApiException.Parse(error)
        }
    }

    private suspend fun HttpResponse.toApiException(): ApiException.Http {
        val text = try {
            bodyAsText()
        } catch (error: CancellationException) {
            throw error
        } catch (_: Exception) {
            ""
        }

        val serverMessage = text
            .takeIf { it.isNotBlank() }
            ?.let { raw ->
                try {
                    json.decodeFromString(ApiErrorDto.serializer(), raw).error
                } catch (_: Exception) {
                    null
                }
            }

        return ApiException.Http(statusCode = status.value, serverMessage = serverMessage)
    }
}

suspend fun <T> apiResult(block: suspend () -> T): Result<T> = try {
    Result.success(block())
} catch (error: CancellationException) {
    throw error
} catch (error: ApiException) {
    Result.failure(error)
} catch (error: Exception) {
    Result.failure(ApiException.Unexpected(error))
}
