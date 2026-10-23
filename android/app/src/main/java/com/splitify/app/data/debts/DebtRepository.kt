package com.splitify.app.data.debts

import com.splitify.app.data.dto.CreateDebtRequest
import com.splitify.app.data.dto.IdResponse
import com.splitify.app.data.dto.SharedDebtDto
import com.splitify.app.data.dto.UserShareDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.builtins.serializer

class DebtRepository(private val api: ApiClient) {

    suspend fun listDebts(groupId: Long): Result<List<SharedDebtDto>> = apiResult {
        api.get("groups/$groupId/debts", ListSerializer(SharedDebtDto.serializer()))
    }

    suspend fun getDebt(debtId: Long): Result<SharedDebtDto> = apiResult {
        api.get("debts/$debtId", SharedDebtDto.serializer())
    }

    suspend fun createDebt(
        groupId: Long,
        name: String,
        amount: String,
        memberIds: List<Long>,
    ): Result<Long> = apiResult {
        api.post(
            path = "groups/$groupId/debts",
            body = CreateDebtRequest(name = name, amount = amount, memberIds = memberIds),
            bodySerializer = CreateDebtRequest.serializer(),
            serializer = IdResponse.serializer(),
        ).id
    }

    suspend fun updateDebt(
        debtId: Long,
        name: String,
        amount: String,
        memberIds: List<Long>,
    ): Result<Unit> = apiResult {
        api.patch(
            path = "debts/$debtId",
            body = CreateDebtRequest(name = name, amount = amount, memberIds = memberIds),
            bodySerializer = CreateDebtRequest.serializer(),
        )
    }

    suspend fun deleteDebt(debtId: Long): Result<Unit> = apiResult {
        api.delete("debts/$debtId")
    }

    suspend fun getDebtMembers(debtId: Long): Result<List<Long>> = apiResult {
        api.get("debts/$debtId/members", ListSerializer(Long.serializer()))
    }

    suspend fun getDebtShares(debtId: Long): Result<List<UserShareDto>> = apiResult {
        api.get("debts/$debtId/shares", ListSerializer(UserShareDto.serializer()))
    }
}
