package com.splitify.app.di

import android.content.Context
import com.splitify.app.BuildConfig
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.auth.TokenStore
import com.splitify.app.data.debts.DebtRepository
import com.splitify.app.data.groups.GroupRepository
import com.splitify.app.data.invites.InviteRepository
import com.splitify.app.data.maps.MapMarkerRepository
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.recurring.RecurringDebtRepository
import com.splitify.app.data.shoppinglists.ShoppingListRepository
import com.splitify.app.data.transactions.TransactionRepository
import kotlinx.coroutines.flow.first

class AppContainer(context: Context) {

    val tokenStore: TokenStore = TokenStore(context)

    private val apiClient: ApiClient = ApiClient(BuildConfig.API_BASE_URL) {
        tokenStore.token.first()
    }

    val authRepository: AuthRepository = AuthRepository(apiClient, tokenStore)

    val groupRepository: GroupRepository = GroupRepository(apiClient)

    val debtRepository: DebtRepository = DebtRepository(apiClient)

    val transactionRepository: TransactionRepository = TransactionRepository(apiClient)

    val inviteRepository: InviteRepository = InviteRepository(apiClient)

    val shoppingListRepository: ShoppingListRepository = ShoppingListRepository(apiClient)

    val recurringDebtRepository: RecurringDebtRepository = RecurringDebtRepository(apiClient)

    val mapMarkerRepository: MapMarkerRepository = MapMarkerRepository(apiClient)
}
