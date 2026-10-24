package com.splitify.app.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.statusBars
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.rememberNavController
import com.splitify.app.data.sync.SyncStatus
import com.splitify.app.di.AppContainer
import com.splitify.app.ui.auth.AuthViewModel
import com.splitify.app.ui.auth.SessionState
import com.splitify.app.ui.components.LoadingScreen
import com.splitify.app.ui.components.SyncBanner
import com.splitify.app.ui.components.isSyncBannerVisible
import com.splitify.app.ui.navigation.AppNavHost
import com.splitify.app.ui.navigation.Routes
import kotlinx.coroutines.delay

@Composable
fun AppRoot(container: AppContainer) {
    val authViewModel: AuthViewModel = viewModel(
        factory = AuthViewModel.factory(
            authRepository = container.authRepository,
            sessionManager = container.session,
            serverConfig = container.serverConfig,
        ),
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

    val authenticated = authState.session is SessionState.Authenticated
    val startDestination = if (authenticated) Routes.GROUPS else Routes.LOGIN
    val username = (authState.session as? SessionState.Authenticated)?.user?.username

    val rawSyncStatus by container.syncManager.status.collectAsStateWithLifecycle()
    val pendingCount by container.localStore.pendingCount.collectAsStateWithLifecycle()
    val isOnline by container.connectivity.isOnline.collectAsStateWithLifecycle()

    // Navigation triggers a background sync. A short "Syncing…" flash on every
    // screen change is noise, so only reveal it once a run actually takes a
    // noticeable amount of time. Offline/error/pending states are immediate.
    var showSyncing by remember { mutableStateOf(false) }
    LaunchedEffect(rawSyncStatus) {
        if (rawSyncStatus is SyncStatus.Syncing) {
            delay(700)
            showSyncing = true
        } else {
            showSyncing = false
        }
    }
    val syncStatus = if (rawSyncStatus is SyncStatus.Syncing && !showSyncing) {
        SyncStatus.Idle
    } else {
        rawSyncStatus
    }
    val showSyncBanner = authenticated && isSyncBannerVisible(syncStatus, pendingCount, isOnline)

    Column(modifier = Modifier.fillMaxSize()) {
        if (authenticated) {
            SyncBanner(
                status = syncStatus,
                pendingCount = pendingCount,
                isOnline = isOnline,
                onSyncNow = container.syncManager::requestSync,
            )
        }
        Box(
            modifier = Modifier
                .weight(1f)
                // The banner sits above the app bar and already covers the status
                // bar area, so the screens below must not inset for it again.
                .then(
                    if (showSyncBanner) {
                        Modifier.consumeWindowInsets(WindowInsets.statusBars)
                    } else {
                        Modifier
                    },
                ),
        ) {
            AppNavHost(
                navController = navController,
                container = container,
                authViewModel = authViewModel,
                startDestination = startDestination,
                username = username,
            )
        }
    }
}
