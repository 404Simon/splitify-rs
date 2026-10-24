package com.splitify.app.data.config

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.emptyPreferences
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import java.io.IOException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch

private val Context.serverDataStore: DataStore<Preferences> by
    preferencesDataStore(name = "splitify_server")

/**
 * The Splitify server the app talks to.
 *
 * There is no compiled-in address: the user enters it on the sign-in screen and
 * it is remembered here, so the same APK works against any deployment. The value
 * is mirrored into a [StateFlow] because [com.splitify.app.data.network.ApiClient]
 * needs it for every request and invite links need it while composing.
 */
class ServerConfig(context: Context, scope: CoroutineScope) {

    private val dataStore: DataStore<Preferences> = context.applicationContext.serverDataStore

    private val _baseUrl = MutableStateFlow<String?>(null)
    val baseUrl: StateFlow<String?> = _baseUrl.asStateFlow()

    private val _isReady = MutableStateFlow(false)
    val isReady: StateFlow<Boolean> = _isReady.asStateFlow()

    init {
        scope.launch {
            dataStore.safeData()
                .map { preferences -> preferences[BASE_URL_KEY] }
                .collect { url ->
                    _baseUrl.value = url?.takeIf { it.isNotBlank() }
                    _isReady.value = true
                }
        }
    }

    suspend fun awaitReady() {
        isReady.first { it }
    }

    /** The configured base URL, or null when the user has not signed in yet. */
    suspend fun current(): String? {
        awaitReady()
        return baseUrl.value
    }

    suspend fun save(baseUrl: String) {
        awaitReady()
        dataStore.edit { preferences -> preferences[BASE_URL_KEY] = baseUrl }
        _baseUrl.value = baseUrl
    }

    private fun DataStore<Preferences>.safeData(): kotlinx.coroutines.flow.Flow<Preferences> =
        data.catch { error ->
            if (error is IOException) emit(emptyPreferences()) else throw error
        }

    private companion object {
        val BASE_URL_KEY = stringPreferencesKey("base_url")
    }
}
