package com.splitify.app.data.groups

import com.splitify.app.data.auth.SessionManager
import com.splitify.app.data.cache.LocalStore
import com.splitify.app.data.dto.GroupDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.GroupSummaryDto
import com.splitify.app.data.sync.SyncManager
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class GroupRepository(
    private val localStore: LocalStore,
    private val syncManager: SyncManager,
    private val session: SessionManager,
) {

    val isLoaded: kotlinx.coroutines.flow.StateFlow<Boolean> = localStore.isLoaded

    fun observeGroups(): Flow<List<GroupSummaryDto>> = localStore.groups

    fun observeGroup(groupId: Long): Flow<GroupDto?> =
        localStore.observeGroup(groupId).map { it?.group }

    fun observeMembers(groupId: Long): Flow<List<GroupMemberDto>> =
        localStore.observeGroup(groupId).map { snapshot ->
            snapshot?.members.orEmpty().sortedWith(
                compareByDescending<GroupMemberDto> { it.isCreator }.thenBy { it.username.lowercase() },
            )
        }

    suspend fun createGroup(name: String): Result<Long> = runCatching {
        val user = session.requireUser()
        val group = localStore.createGroup(name.trim(), user)
        syncManager.requestSync()
        group.id
    }

    fun refresh() {
        syncManager.requestSync()
    }
}
