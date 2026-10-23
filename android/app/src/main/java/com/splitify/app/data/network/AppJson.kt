package com.splitify.app.data.network

import kotlinx.serialization.json.Json

/**
 * One JSON configuration for the wire format, the offline cache and the outbox
 * payloads. Keeping them identical avoids subtle encode/decode mismatches.
 */
val AppJson: Json = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    encodeDefaults = true
}
