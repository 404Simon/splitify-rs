package com.splitify.app.data.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Everything the client needs for one group, fetched with a single request.
 *
 * Balances are intentionally absent: they are derived locally from [debts] and
 * [transactions] so they stay correct while offline (see BalancesCalculator).
 */
@Serializable
data class GroupSnapshotDto(
    val group: GroupDto,
    val members: List<GroupMemberDto> = emptyList(),
    val debts: List<DebtSnapshotDto> = emptyList(),
    val transactions: List<TransactionDto> = emptyList(),
    @SerialName("shopping_lists") val shoppingLists: List<ShoppingListSnapshotDto> = emptyList(),
    @SerialName("recurring_debts") val recurringDebts: List<RecurringDebtSnapshotDto> = emptyList(),
    val markers: List<MapMarkerDto> = emptyList(),
    val invites: List<InviteListItemDto> = emptyList(),
    @SerialName("generated_at") val generatedAt: String = "",
)

@Serializable
data class DebtSnapshotDto(
    val debt: SharedDebtDto,
    @SerialName("participant_ids") val participantIds: List<Long> = emptyList(),
    val shares: List<UserShareDto> = emptyList(),
)

@Serializable
data class ShoppingListSnapshotDto(
    val list: ShoppingListDto,
    val items: List<ShoppingListItemDto> = emptyList(),
    val activity: List<ShoppingListActivityDto> = emptyList(),
)

@Serializable
data class RecurringDebtSnapshotDto(
    val debt: RecurringDebtDto,
    @SerialName("member_ids") val memberIds: List<Long> = emptyList(),
    val shares: List<RecurringDebtMemberDto> = emptyList(),
    val instances: List<GeneratedInstanceDto> = emptyList(),
)

/** Explicit desired state for shopping-item toggles (makes retries idempotent). */
@Serializable
data class ToggleItemRequest(
    @SerialName("is_completed") val isCompleted: Boolean,
)

/** Explicit desired state for recurring-debt toggles (makes retries idempotent). */
@Serializable
data class ToggleRecurringRequest(
    @SerialName("is_active") val isActive: Boolean,
)
