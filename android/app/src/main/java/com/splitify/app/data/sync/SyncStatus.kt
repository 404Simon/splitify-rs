package com.splitify.app.data.sync

/** Coarse sync state for the global banner. Pending counts live in LocalStore. */
sealed interface SyncStatus {
    data object Idle : SyncStatus
    data object Syncing : SyncStatus
    data object Offline : SyncStatus
    data class Error(val message: String) : SyncStatus
}
