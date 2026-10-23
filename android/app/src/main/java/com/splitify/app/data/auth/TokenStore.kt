package com.splitify.app.data.auth

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.emptyPreferences
import androidx.datastore.preferences.core.longPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.splitify.app.data.dto.UserDto
import java.io.IOException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext

private val Context.sessionDataStore: DataStore<Preferences> by preferencesDataStore(name = "splitify_session")

/**
 * Persists the bearer token (Keystore-encrypted) and the cached user profile.
 *
 * Caching the user is what makes an offline launch possible: the app can render
 * the authenticated UI before it has talked to the server.
 */
class TokenStore(context: Context) {

    private val dataStore: DataStore<Preferences> = context.applicationContext.sessionDataStore

    private val cipher: TokenCipher = TokenCipher()

    val token: Flow<String?> = dataStore.safeData()
        .map { it[TOKEN_KEY] }
        .map { encrypted -> encrypted?.let { withContext(Dispatchers.IO) { cipher.decrypt(it) } } }

    val user: Flow<UserDto?> = dataStore.safeData().map { preferences ->
        val id = preferences[USER_ID_KEY]
        val username = preferences[USERNAME_KEY]
        if (id != null && !username.isNullOrBlank()) UserDto(id = id, username = username) else null
    }

    suspend fun save(token: String, user: UserDto) {
        val encryptedToken = withContext(Dispatchers.IO) { cipher.encrypt(token) }
        dataStore.edit { preferences ->
            preferences[TOKEN_KEY] = encryptedToken
            preferences[USER_ID_KEY] = user.id
            preferences[USERNAME_KEY] = user.username
        }
    }

    suspend fun saveUser(user: UserDto) {
        dataStore.edit { preferences ->
            preferences[USER_ID_KEY] = user.id
            preferences[USERNAME_KEY] = user.username
        }
    }

    suspend fun clear() {
        dataStore.edit { preferences ->
            preferences.remove(TOKEN_KEY)
            preferences.remove(USER_ID_KEY)
            preferences.remove(USERNAME_KEY)
        }
    }

    private fun DataStore<Preferences>.safeData(): Flow<Preferences> =
        data.catch { error ->
            if (error is IOException) emit(emptyPreferences()) else throw error
        }

    private companion object {
        val TOKEN_KEY = stringPreferencesKey("token")
        val USER_ID_KEY = longPreferencesKey("user_id")
        val USERNAME_KEY = stringPreferencesKey("username")
    }
}
