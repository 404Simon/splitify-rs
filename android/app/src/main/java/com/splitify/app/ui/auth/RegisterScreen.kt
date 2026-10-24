package com.splitify.app.ui.auth

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Email
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.splitify.app.ui.components.ErrorBanner

@Composable
fun RegisterScreen(
    viewModel: AuthViewModel,
    onNavigateToLogin: () -> Unit,
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    var username by rememberSaveable { mutableStateOf("") }
    var email by rememberSaveable { mutableStateOf("") }
    var password by rememberSaveable { mutableStateOf("") }
    var confirmPassword by rememberSaveable { mutableStateOf("") }

    LaunchedEffect(Unit) { viewModel.clearError() }

    val canSubmit = username.isNotBlank() &&
        password.isNotBlank() &&
        confirmPassword.isNotBlank() &&
        !uiState.isSubmitting

    val submit = {
        if (canSubmit) {
            viewModel.register(
                username = username,
                email = email,
                password = password,
                confirmPassword = confirmPassword,
            )
        }
    }

    AuthScaffold(
        title = "Create your account",
        subtitle = "One place for shared expenses, balances and plans with your groups.",
    ) {
        AuthTextField(
            value = uiState.serverUrl,
            onValueChange = viewModel::setServerUrl,
            label = "Server URL",
            icon = Icons.Default.Settings,
            enabled = !uiState.isSubmitting,
            supportingText = "e.g. https://splitify.example.com",
            keyboardOptions = KeyboardOptions(
                keyboardType = KeyboardType.Uri,
                imeAction = ImeAction.Next,
            ),
        )

        AuthTextField(
            value = username,
            onValueChange = { username = it },
            label = "Username",
            icon = Icons.Default.Person,
            enabled = !uiState.isSubmitting,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
        )

        AuthTextField(
            value = email,
            onValueChange = { email = it },
            label = "Email (optional)",
            icon = Icons.Default.Email,
            enabled = !uiState.isSubmitting,
            keyboardOptions = KeyboardOptions(
                keyboardType = KeyboardType.Email,
                imeAction = ImeAction.Next,
            ),
        )

        AuthTextField(
            value = password,
            onValueChange = { password = it },
            label = "Password",
            icon = Icons.Default.Lock,
            enabled = !uiState.isSubmitting,
            supportingText = "At least 8 characters",
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(
                keyboardType = KeyboardType.Password,
                imeAction = ImeAction.Next,
            ),
        )

        AuthTextField(
            value = confirmPassword,
            onValueChange = { confirmPassword = it },
            label = "Confirm password",
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
            text = "Create account",
            isLoading = uiState.isSubmitting,
            enabled = canSubmit,
            onClick = submit,
        )

        AuthFooterLink(
            text = "Already have an account? Sign in",
            enabled = !uiState.isSubmitting,
            onClick = onNavigateToLogin,
        )
    }
}
