package com.splitify.app.data.config

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ServerUrlTest {

    @Test
    fun `bare host defaults to https and appends the api prefix`() {
        assertEquals(
            "https://splitify.example.com/api/v1/",
            ServerUrl.normalize("splitify.example.com"),
        )
    }

    @Test
    fun `explicit scheme host and port are kept`() {
        assertEquals(
            "http://10.0.2.2:3000/api/v1/",
            ServerUrl.normalize("http://10.0.2.2:3000"),
        )
    }

    @Test
    fun `trailing slash and existing api prefix are normalized`() {
        assertEquals(
            "https://api.example.com/api/v1/",
            ServerUrl.normalize("  https://api.example.com/api/v1/  "),
        )
    }

    @Test
    fun `reverse proxy prefix is preserved`() {
        assertEquals(
            "https://example.com/splitify/api/v1/",
            ServerUrl.normalize("https://example.com/splitify/api/v1"),
        )
        assertEquals(
            "https://example.com/splitify/api/v1/",
            ServerUrl.normalize("https://example.com/splitify"),
        )
    }

    @Test
    fun `unsupported or unsafe inputs are rejected`() {
        assertNull(ServerUrl.normalize(""))
        assertNull(ServerUrl.normalize("   "))
        assertNull(ServerUrl.normalize("ftp://example.com"))
        assertNull(ServerUrl.normalize("https://user:secret@example.com"))
        assertNull(ServerUrl.normalize("https://example.com?token=1"))
        assertNull(ServerUrl.normalize("not a url"))
    }

    @Test
    fun `release builds reject cleartext but accept https`() {
        assertEquals(
            "This build only allows HTTPS server addresses.",
            ServerUrl.errorFor("http://example.com", allowCleartext = false),
        )
        assertNull(ServerUrl.errorFor("https://example.com", allowCleartext = false))
    }

    @Test
    fun `debug builds allow cleartext`() {
        assertNull(ServerUrl.errorFor("http://10.0.2.2:3000", allowCleartext = true))
    }

    @Test
    fun `blank input reports a format error`() {
        assertEquals(
            "Enter your server address, e.g. https://splitify.example.com",
            ServerUrl.errorFor("", allowCleartext = true),
        )
    }
}
