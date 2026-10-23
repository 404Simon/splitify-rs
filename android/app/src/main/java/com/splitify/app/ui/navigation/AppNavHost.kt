package com.splitify.app.ui.navigation

import androidx.compose.runtime.Composable
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.navArgument
import com.splitify.app.di.AppContainer
import com.splitify.app.ui.auth.AuthViewModel
import com.splitify.app.ui.auth.LoginScreen
import com.splitify.app.ui.auth.RegisterScreen
import com.splitify.app.ui.groups.GroupDetailScreen
import com.splitify.app.ui.groups.GroupDetailViewModel
import com.splitify.app.ui.groups.GroupsScreen
import com.splitify.app.ui.groups.GroupsViewModel

object Routes {
    const val LOGIN = "login"
    const val REGISTER = "register"
    const val GROUPS = "groups"
    const val GROUP_DETAIL = "groups/{groupId}"

    fun groupDetail(groupId: Long): String = "groups/$groupId"
}

@Composable
fun AppNavHost(
    navController: NavHostController,
    container: AppContainer,
    authViewModel: AuthViewModel,
    startDestination: String,
    username: String?,
) {
    NavHost(
        navController = navController,
        startDestination = startDestination,
    ) {
        composable(Routes.LOGIN) {
            LoginScreen(
                viewModel = authViewModel,
                onNavigateToRegister = {
                    navController.navigate(Routes.REGISTER) { launchSingleTop = true }
                },
            )
        }

        composable(Routes.REGISTER) {
            RegisterScreen(
                viewModel = authViewModel,
                onNavigateToLogin = { navController.popBackStack() },
            )
        }

        composable(Routes.GROUPS) {
            val groupsViewModel: GroupsViewModel = viewModel(
                factory = GroupsViewModel.factory(
                    groupRepository = container.groupRepository,
                    authRepository = container.authRepository,
                    inviteRepository = container.inviteRepository,
                ),
            )
            GroupsScreen(
                viewModel = groupsViewModel,
                username = username,
                onOpenGroup = { groupId ->
                    navController.navigate(Routes.groupDetail(groupId))
                },
            )
        }

        composable(
            route = Routes.GROUP_DETAIL,
            arguments = listOf(navArgument("groupId") { type = NavType.LongType }),
        ) { entry ->
            val groupId = entry.arguments?.getLong("groupId") ?: return@composable
            val detailViewModel: GroupDetailViewModel = viewModel(
                key = "group-$groupId",
                factory = GroupDetailViewModel.factory(
                    groupId = groupId,
                    groupRepository = container.groupRepository,
                    authRepository = container.authRepository,
                ),
            )
            GroupDetailScreen(
                viewModel = detailViewModel,
                debtRepository = container.debtRepository,
                transactionRepository = container.transactionRepository,
                inviteRepository = container.inviteRepository,
                shoppingListRepository = container.shoppingListRepository,
                recurringDebtRepository = container.recurringDebtRepository,
                mapMarkerRepository = container.mapMarkerRepository,
                authRepository = container.authRepository,
                onBack = { navController.popBackStack() },
            )
        }
    }
}
