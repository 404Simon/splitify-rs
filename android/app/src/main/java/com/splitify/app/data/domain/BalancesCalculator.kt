package com.splitify.app.data.domain

import com.splitify.app.data.dto.DebtRelationshipDto
import com.splitify.app.data.dto.DebtSnapshotDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.TransactionDto
import com.splitify.app.data.dto.UserBalanceDto
import java.math.BigDecimal
import java.math.MathContext
import java.math.RoundingMode

/**
 * Computes the same balances as the server's `group_balances`, from local data.
 * Keeping this client-side is what makes the Balances tab usable offline.
 */
object BalancesCalculator {

    private val DIVISION = MathContext(28, RoundingMode.HALF_UP)

    fun calculate(
        members: List<GroupMemberDto>,
        debts: List<DebtSnapshotDto>,
        transactions: List<TransactionDto>,
    ): List<UserBalanceDto> {
        if (members.isEmpty()) return emptyList()

        val contributions = members.associate { it.id to mutableMapOf<Long, BigDecimal>() }

        for (entry in debts) {
            val participants = entry.participantIds
            if (participants.isEmpty()) continue
            val amount = entry.debt.amount.toBigDecimalOrNull() ?: continue
            val share = amount.divide(BigDecimal(participants.size), DIVISION)
            val creatorId = entry.debt.createdBy

            for (participantId in participants) {
                if (participantId == creatorId) continue
                contributions[participantId]?.merge(creatorId, share, BigDecimal::add)
                contributions[creatorId]?.merge(participantId, share.negate(), BigDecimal::add)
            }
        }

        for (transaction in transactions) {
            val amount = transaction.amount.toBigDecimalOrNull() ?: continue
            contributions[transaction.payerId]?.merge(
                transaction.recipientId,
                amount.negate(),
                BigDecimal::add,
            )
            contributions[transaction.recipientId]?.merge(
                transaction.payerId,
                amount,
                BigDecimal::add,
            )
        }

        val usernames = members.associate { it.id to it.username }

        return members.map { member ->
            val userDebts = contributions[member.id].orEmpty()
            var totalOwed = BigDecimal.ZERO
            var totalOwing = BigDecimal.ZERO
            val relationships = userDebts
                .filterValues { it.compareTo(BigDecimal.ZERO) != 0 }
                .map { (otherId, amount) ->
                    val positive = amount > BigDecimal.ZERO
                    if (positive) totalOwing += amount else totalOwed += amount.abs()
                    DebtRelationshipDto(
                        otherUserId = otherId,
                        otherUsername = usernames[otherId] ?: "",
                        amount = amount.abs().display(),
                        relationshipType = if (positive) "Owes" else "Owed",
                    )
                }
                .sortedBy { it.otherUsername.lowercase() }

            val net = totalOwed - totalOwing
            UserBalanceDto(
                userId = member.id,
                username = member.username,
                relationships = relationships,
                totalOwed = totalOwed.display(),
                totalOwing = totalOwing.display(),
                netAmount = net.abs().display(),
                netType = when {
                    net > BigDecimal.ZERO -> "Positive"
                    net < BigDecimal.ZERO -> "Negative"
                    else -> "Neutral"
                },
            )
        }
    }

    private fun BigDecimal.display(): String =
        setScale(2, RoundingMode.HALF_UP).toPlainString()

    private fun String.toBigDecimalOrNull(): BigDecimal? =
        runCatching { BigDecimal(this) }.getOrNull()
}
