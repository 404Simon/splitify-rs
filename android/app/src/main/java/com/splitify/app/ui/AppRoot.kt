package com.splitify.app.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.rememberNavController
import com.splitify.app.di.AppContainer
import com.splitify.app.ui.auth.AuthViewModel
import com.splitify.app.ui.auth.SessionState
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.navigation.AppNavHost
import com.splitify.app.ui.navigation.Routes

@Composable
fun AppRoot(container: AppContainer) {
    val authViewModel: AuthViewModel = viewModel(
        factory = AuthViewModel.factory(container.authRepository),
    )
    val authState by authViewModel.uiState.collectAsStateWithLifecycle()
    val navController = rememberNavController()

    LaunchedEffect(authState.session) {
        when (authState.session) {
            SessionState.Checking -> Unit

            SessionState.Unauthenticated -> navController.navigate(Routes.LOGIN) {
                popUpTo(navController.graph.id) { inclusive = true }
                launchSingleTop = true
            }

            is SessionState.Authenticated -> navController.navigate(Routes.GROUPS) {
                popUpTo(navController.graph.id) { inclusive = true }
                launchSingleTop = true
            }
        }
    }

    if (authState.session is SessionState.Checking) {
        LoadingScreen(message = "Checking your session…")
        return
    }

    val startDestination = if (authState.session is SessionState.Authenticated) {
        Routes.GROUPS
    } else {
        Routes.LOGIN
    }

    AppNavHost(
        navController = navController,
        container = container,
        authViewModel = authViewModel,
        startDestination = startDestination,
        username = (authState.session as? SessionState.Authenticated)?.user?.username,
    )
}
