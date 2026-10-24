package com.splitify.app.data.network

sealed class ApiException(message: String, cause: Throwable? = null) : Exception(message, cause) {

    data class Http(val statusCode: Int, val serverMessage: String?) : ApiException(
        serverMessage?.takeIf { it.isNotBlank() } ?: "Request failed ($statusCode)",
    ) {
        val isUnauthorized: Boolean get() = statusCode == 401
    }

    class Unauthorized : ApiException("Your session has expired. Please sign in again.")

    class NotConfigured :
        ApiException("No server address is set. Sign in with your server address.")

    class Network(cause: Throwable) :
        ApiException("Could not reach the server. Check your connection.", cause)

    class Parse(cause: Throwable) :
        ApiException("The server returned an unexpected response.", cause)

    class Unexpected(cause: Throwable) :
        ApiException("Something went wrong. Please try again.", cause)
}

fun Throwable.userMessage(): String =
    if (this is ApiException) message ?: "Something went wrong. Please try again."
    else "Something went wrong. Please try again."
