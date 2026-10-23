package com.splitify.app.data.auth

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.emptyPreferences
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import java.io.IOException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext

private val Context.sessionDataStore: DataStore<Preferences> by preferencesDataStore(name = "splitify_session")

class TokenStore(context: Context) {

    private val dataStore: DataStore<Preferences> = context.applicationContext.sessionDataStore

    private val cipher: TokenCipher = TokenCipher()

    val token: Flow<String?> = dataStore.safeData()
        .map { it[TOKEN_KEY] }
        .map { encrypted -> encrypted?.let { withContext(Dispatchers.IO) { cipher.decrypt(it) } } }

    val username: Flow<String?> = dataStore.safeData().map { it[USERNAME_KEY] }

    suspend fun save(token: String, username: String) {
        val encryptedToken = withContext(Dispatchers.IO) { cipher.encrypt(token) }
        dataStore.edit { preferences ->
            preferences[TOKEN_KEY] = encryptedToken
            preferences[USERNAME_KEY] = username
        }
    }

    suspend fun clear() {
        dataStore.edit { preferences ->
            preferences.remove(TOKEN_KEY)
            preferences.remove(USERNAME_KEY)
        }
    }

    private fun DataStore<Preferences>.safeData(): Flow<Preferences> =
        data.catch { error ->
            if (error is IOException) emit(emptyPreferences()) else throw error
        }

    private companion object {
        val TOKEN_KEY = stringPreferencesKey("token")
        val USERNAME_KEY = stringPreferencesKey("username")
    }
}
