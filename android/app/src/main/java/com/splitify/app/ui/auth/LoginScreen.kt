package com.splitify.app.ui.auth

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Person
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.splitify.app.ui.components.ErrorBanner

@Composable
fun LoginScreen(
    viewModel: AuthViewModel,
    onNavigateToRegister: () -> Unit,
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    var username by rememberSaveable { mutableStateOf("") }
    var password by rememberSaveable { mutableStateOf("") }

    LaunchedEffect(Unit) { viewModel.clearError() }

    val canSubmit = username.isNotBlank() && password.isNotBlank() && !uiState.isSubmitting
    val submit = { if (canSubmit) viewModel.login(username, password) }

    AuthScaffold(
        title = "Welcome back",
        subtitle = "Sign in to split expenses with your people — no spreadsheet required.",
    ) {
        AuthTextField(
            value = username,
            onValueChange = { username = it },
            label = "Username",
            icon = Icons.Default.Person,
            enabled = !uiState.isSubmitting,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
        )

        AuthTextField(
            value = password,
            onValueChange = { password = it },
            label = "Password",
            icon = Icons.Default.Lock,
            enabled = !uiState.isSubmitting,
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(
                keyboardType = KeyboardType.Password,
                imeAction = ImeAction.Done,
            ),
            keyboardActions = KeyboardActions(onDone = { submit() }),
        )

        uiState.error?.let { message ->
            ErrorBanner(message = message)
        }

        Spacer(Modifier.height(2.dp))

        AuthPrimaryButton(
            text = "Sign in",
            isLoading = uiState.isSubmitting,
            enabled = canSubmit,
            onClick = submit,
        )

        AuthFooterLink(
            text = "New here? Create an account",
            enabled = !uiState.isSubmitting,
            onClick = onNavigateToRegister,
        )
    }
}
