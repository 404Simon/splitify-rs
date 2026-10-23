package com.splitify.app.di

import android.content.Context
import com.splitify.app.BuildConfig
import com.splitify.app.data.auth.AuthRepository
import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.auth.TokenStore
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.debts.DebtRepository
import com.splitify.app.data.groups.GroupRepository
import com.splitify.app.data.invites.InviteRepository
import com.splitify.app.data.local.SplitifyDatabase
import com.splitify.app.data.maps.MapMarkerRepository
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.SyncApi
import com.splitify.app.data.recurring.RecurringDebtRepository
import com.splitify.app.data.shoppinglists.ShoppingListRepository
import com.splitify.app.data.sync.ConnectivityObserver
import com.splitify.app.data.sync.SyncManager
import com.splitify.app.data.sync.SyncScheduler
import com.splitify.app.data.transactions.TransactionRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

/**
 * Manual dependency graph. The app is small enough that an explicit container
 * is easier to follow than a DI framework, and it keeps startup predictable.
 */
class AppContainer(context: Context) {

    private val appContext = context.applicationContext
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    val tokenStore: TokenStore = TokenStore(appContext)

    val session: SessionManager = SessionManager(tokenStore, scope)

    private val apiClient: ApiClient = ApiClient(
        baseUrl = BuildConfig.API_BASE_URL,
        tokenProvider = { session.token.value },
        onUnauthorized = { session.clear() },
    )

    private val database: SplitifyDatabase = SplitifyDatabase.build(appContext)

    val localStore: LocalStore = LocalStore(database, scope)

    val connectivity: ConnectivityObserver = ConnectivityObserver(appContext)

    private val syncApi: SyncApi = SyncApi(apiClient)

    val syncManager: SyncManager = SyncManager(
        api = syncApi,
        store = localStore,
        session = session,
        connectivity = connectivity,
        scope = scope,
        scheduleRetry = { SyncScheduler.request(appContext) },
    )

    val authRepository: AuthRepository = AuthRepository(apiClient, session, localStore)

    val groupRepository: GroupRepository = GroupRepository(localStore, syncManager, session)

    val debtRepository: DebtRepository = DebtRepository(localStore, syncManager, session)

    val transactionRepository: TransactionRepository =
        TransactionRepository(localStore, syncManager, session)

    val inviteRepository: InviteRepository = InviteRepository(apiClient, localStore, syncManager)

    val shoppingListRepository: ShoppingListRepository =
        ShoppingListRepository(localStore, syncManager, session)

    val recurringDebtRepository: RecurringDebtRepository =
        RecurringDebtRepository(localStore, syncManager, session)

    val mapMarkerRepository: MapMarkerRepository =
        MapMarkerRepository(localStore, syncManager, session)

    init {
        scope.launch {
            localStore.load()
            syncManager.start()
            session.awaitReady()
            if (session.token.value != null) syncManager.requestSync()
        }
        SyncScheduler.schedulePeriodic(appContext)
    }
}
