package com.splitify.app.data.config

import java.net.URI

/**
 * Parsing and validation for the server address typed by the user.
 *
 * The stored form is always a base URL that ends with `/api/v1/`, so every API
 * path can simply be appended to it.
 */
object ServerUrl {

    const val API_PREFIX: String = "/api/v1"

    private const val DEFAULT_SCHEME = "https://"

    private const val FORMAT_ERROR =
        "Enter your server address, e.g. https://splitify.example.com"
    private const val HTTPS_ERROR = "This build only allows HTTPS server addresses."

    /**
     * Normalizes user input into a base URL (with a trailing slash).
     *
     * `splitify.example.com` becomes `https://splitify.example.com/api/v1/`.
     * A non-empty path is kept as-is, so a server mounted under a reverse-proxy
     * prefix still works. Returns null for anything that is not a usable
     * http(s) URL without credentials, query or fragment.
     */
    fun normalize(input: String): String? {
        val trimmed = input.trim()
        if (trimmed.isEmpty() || trimmed.any { it.isWhitespace() }) return null

        val withScheme = if (trimmed.contains("://")) trimmed else DEFAULT_SCHEME + trimmed
        val uri = try {
            URI(withScheme)
        } catch (_: Exception) {
            return null
        }

        val scheme = uri.scheme?.lowercase() ?: return null
        if (scheme != "http" && scheme != "https") return null
        if (uri.userInfo != null || uri.query != null || uri.fragment != null) return null

        // rawAuthority keeps an explicit port and IPv6 brackets intact.
        val authority = uri.rawAuthority ?: return null
        if (authority.isBlank()) return null

        val path = uri.rawPath.orEmpty().trimEnd('/')
        val normalizedPath = when {
            path.isEmpty() -> API_PREFIX
            path.endsWith(API_PREFIX) -> path
            // The API always lives under /api/v1, so a reverse-proxy prefix only
            // needs the mount point.
            else -> path + API_PREFIX
        }
        return "$scheme://$authority$normalizedPath/"
    }

    /** Null when [input] can be used for API calls in this build. */
    fun errorFor(input: String, allowCleartext: Boolean): String? {
        val normalized = normalize(input) ?: return FORMAT_ERROR
        if (!allowCleartext && !normalized.startsWith("https://")) return HTTPS_ERROR
        return null
    }
}
