package com.splitify.app.data.domain

import com.splitify.app.data.dto.DebtSnapshotDto
import com.splitify.app.data.dto.GroupMemberDto
import com.splitify.app.data.dto.SharedDebtDto
import com.splitify.app.data.dto.TransactionDto
import com.splitify.app.data.dto.UserBalanceDto
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Parity fixtures for the server's `group_balances` implementation:
 * - every non-creator participant owes `amount / participantCount` to the creator;
 * - a transaction decreases the payer's map entry and increases the recipient's;
 * - positive map values mean "Owes", negative values mean "Owed".
 */
class BalancesCalculatorTest {

    @Test
    fun `shared debt splits equally among non-creator participants`() {
        val members = listOf(member(1, "alice"), member(2, "bob"), member(3, "carol"))

        val balances = BalancesCalculator.calculate(
            members = members,
            debts = listOf(debt(createdBy = 1, amount = "30.00", participants = listOf(1, 2, 3))),
            transactions = emptyList(),
        )

        val alice = balances.byUser(1)
        assertEquals("20.00", alice.totalOwed)
        assertEquals("0.00", alice.totalOwing)
        assertEquals("20.00", alice.netAmount)
        assertEquals("Positive", alice.netType)
        assertEquals(2, alice.relationships.size)
        alice.relationships.forEach { relationship ->
            assertEquals("Owed", relationship.relationshipType)
            assertEquals("10.00", relationship.amount)
        }

        val bob = balances.byUser(2)
        assertEquals("0.00", bob.totalOwed)
        assertEquals("10.00", bob.totalOwing)
        assertEquals("10.00", bob.netAmount)
        assertEquals("Negative", bob.netType)
        assertEquals(1, bob.relationships.size)
        assertEquals("alice", bob.relationships.single().otherUsername)
        assertEquals("Owes", bob.relationships.single().relationshipType)
        assertEquals("10.00", bob.relationships.single().amount)
    }

    @Test
    fun `transaction decreases payer and increases recipient`() {
        val members = listOf(member(1, "alice"), member(2, "bob"))

        val balances = BalancesCalculator.calculate(
            members = members,
            debts = emptyList(),
            transactions = listOf(transaction(payerId = 1, recipientId = 2, amount = "25.00")),
        )

        val alice = balances.byUser(1)
        assertEquals("25.00", alice.totalOwed)
        assertEquals("0.00", alice.totalOwing)
        assertEquals("Positive", alice.netType)
        val aliceToBob = alice.relationships.single()
        assertEquals(2L, aliceToBob.otherUserId)
        assertEquals("Owed", aliceToBob.relationshipType)
        assertEquals("25.00", aliceToBob.amount)

        val bob = balances.byUser(2)
        assertEquals("0.00", bob.totalOwed)
        assertEquals("25.00", bob.totalOwing)
        assertEquals("Negative", bob.netType)
        val bobToAlice = bob.relationships.single()
        assertEquals(1L, bobToAlice.otherUserId)
        assertEquals("Owes", bobToAlice.relationshipType)
        assertEquals("25.00", bobToAlice.amount)
    }

    @Test
    fun `net is neutral when a debt and its settling transaction cancel out`() {
        val members = listOf(member(1, "alice"), member(2, "bob"))

        val balances = BalancesCalculator.calculate(
            members = members,
            debts = listOf(debt(createdBy = 1, amount = "10.00", participants = listOf(1, 2))),
            transactions = listOf(transaction(payerId = 2, recipientId = 1, amount = "5.00")),
        )

        balances.forEach { balance ->
            assertEquals("0.00", balance.totalOwed)
            assertEquals("0.00", balance.totalOwing)
            assertEquals("0.00", balance.netAmount)
            assertEquals("Neutral", balance.netType)
            assertTrue(balance.relationships.isEmpty())
        }
    }

    @Test
    fun `net positive and negative use absolute display values`() {
        val members = listOf(member(1, "alice"), member(2, "bob"), member(3, "carol"))

        val balances = BalancesCalculator.calculate(
            members = members,
            debts = listOf(
                debt(id = 1, createdBy = 1, amount = "60.00", participants = listOf(1, 2, 3)),
                debt(id = 2, createdBy = 2, amount = "12.00", participants = listOf(2, 3)),
            ),
            transactions = emptyList(),
        )

        // alice: +20 from bob, +20 from carol.
        val alice = balances.byUser(1)
        assertEquals("40.00", alice.totalOwed)
        assertEquals("Positive", alice.netType)
        assertEquals("40.00", alice.netAmount)

        // bob: owes 20 to alice, is owed 6 by carol.
        val bob = balances.byUser(2)
        assertEquals("6.00", bob.totalOwed)
        assertEquals("20.00", bob.totalOwing)
        assertEquals("14.00", bob.netAmount)
        assertEquals("Negative", bob.netType)
        assertEquals(
            "Owes",
            bob.relationships.single { it.otherUserId == 1L }.relationshipType,
        )
        assertEquals(
            "Owed",
            bob.relationships.single { it.otherUserId == 3L }.relationshipType,
        )
    }

    @Test
    fun `shares round half up to two decimals`() {
        val members = listOf(member(1, "alice"), member(2, "bob"), member(3, "carol"))

        val balances = BalancesCalculator.calculate(
            members = members,
            debts = listOf(debt(createdBy = 1, amount = "10.00", participants = listOf(1, 2, 3))),
            transactions = emptyList(),
        )

        // 10 / 3 = 3.3333…; each relationship rounds to 3.33, the creator total to 6.67.
        val bob = balances.byUser(2)
        assertEquals("3.33", bob.relationships.single().amount)
        assertEquals("3.33", bob.totalOwing)
        assertEquals("3.33", bob.netAmount)

        val alice = balances.byUser(1)
        assertEquals("6.67", alice.totalOwed)
        assertEquals("6.67", alice.netAmount)
    }

    @Test
    fun `transaction amounts round half up to two decimals`() {
        val members = listOf(member(1, "alice"), member(2, "bob"))

        val balances = BalancesCalculator.calculate(
            members = members,
            debts = emptyList(),
            transactions = listOf(transaction(payerId = 1, recipientId = 2, amount = "0.005")),
        )

        assertEquals("0.01", balances.byUser(1).relationships.single().amount)
        assertEquals("0.01", balances.byUser(1).netAmount)
        assertEquals("0.01", balances.byUser(2).netAmount)
        assertEquals("Negative", balances.byUser(2).netType)
    }

    @Test
    fun `participant count includes members that are not creators`() {
        val members = listOf(member(1, "alice"), member(2, "bob"))
        val balances = BalancesCalculator.calculate(
            members = members,
            debts = listOf(debt(createdBy = 1, amount = "30.00", participants = listOf(2, 3))),
            transactions = emptyList(),
        )

        // Server divides by the participant count (2) even though the creator is not
        // a participant, and ignores participants that are not group members.
        assertEquals("15.00", balances.byUser(2).totalOwing)
        assertEquals("30.00", balances.byUser(1).totalOwed)
        assertEquals("Positive", balances.byUser(1).netType)
    }

    @Test
    fun `participants that are not group members still count for the divisor`() {
        val members = listOf(member(1, "alice"), member(2, "bob"))

        val balances = BalancesCalculator.calculate(
            members = members,
            debts = listOf(debt(createdBy = 1, amount = "30.00", participants = listOf(1, 2, 99))),
            transactions = emptyList(),
        )

        assertEquals("10.00", balances.byUser(2).totalOwing)
    }

    @Test
    fun `no members yields no balances`() {
        val balances = BalancesCalculator.calculate(
            members = emptyList(),
            debts = listOf(debt(createdBy = 1, amount = "10.00", participants = listOf(1, 2))),
            transactions = listOf(transaction(payerId = 1, recipientId = 2, amount = "5.00")),
        )

        assertTrue(balances.isEmpty())
    }

    private fun List<UserBalanceDto>.byUser(id: Long): UserBalanceDto =
        single { it.userId == id }

    private fun member(id: Long, username: String) =
        GroupMemberDto(id = id, username = username, isCreator = false)

    private fun debt(
        id: Long = 1,
        createdBy: Long,
        amount: String,
        participants: List<Long>,
    ) = DebtSnapshotDto(
        debt = SharedDebtDto(
            id = id,
            groupId = 1,
            createdBy = createdBy,
            creatorUsername = "user$createdBy",
            name = "Dinner",
            amount = amount,
            createdAt = "2026-01-01T00:00:00Z",
            updatedAt = "2026-01-01T00:00:00Z",
            isCreator = false,
        ),
        participantIds = participants,
        shares = emptyList(),
    )

    private fun transaction(
        id: Long = 1,
        payerId: Long,
        recipientId: Long,
        amount: String,
    ) = TransactionDto(
        id = id,
        groupId = 1,
        payerId = payerId,
        payerUsername = "user$payerId",
        recipientId = recipientId,
        recipientUsername = "user$recipientId",
        amount = amount,
        description = null,
        createdAt = "2026-01-01T00:00:00Z",
        updatedAt = "2026-01-01T00:00:00Z",
    )
}
