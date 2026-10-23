package com.splitify.app.data.recurring

import com.splitify.app.data.dto.CreateRecurringDebtRequest
import com.splitify.app.data.dto.GeneratedInstanceDto
import com.splitify.app.data.dto.GenerateRecurringResponse
import com.splitify.app.data.dto.IdResponse
import com.splitify.app.data.dto.RecurringDebtDto
import com.splitify.app.data.dto.RecurringDebtMemberDto
import com.splitify.app.data.dto.ToggleRecurringResponse
import com.splitify.app.data.dto.UpdateRecurringDebtRequest
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.JsonObject

class RecurringDebtRepository(private val api: ApiClient) {

    suspend fun listRecurringDebts(groupId: Long): Result<List<RecurringDebtDto>> = apiResult {
        api.get(
            "groups/$groupId/recurring-debts",
            ListSerializer(RecurringDebtDto.serializer()),
        )
    }

    suspend fun getRecurringDebt(recurringDebtId: Long): Result<RecurringDebtDto> = apiResult {
        api.get("recurring-debts/$recurringDebtId", RecurringDebtDto.serializer())
    }

    suspend fun createRecurringDebt(
        groupId: Long,
        request: CreateRecurringDebtRequest,
    ): Result<Long> = apiResult {
        api.post(
            path = "groups/$groupId/recurring-debts",
            body = request,
            bodySerializer = CreateRecurringDebtRequest.serializer(),
            serializer = IdResponse.serializer(),
        ).id
    }

    suspend fun updateRecurringDebt(
        recurringDebtId: Long,
        request: UpdateRecurringDebtRequest,
    ): Result<Unit> = apiResult {
        api.patch(
            path = "recurring-debts/$recurringDebtId",
            body = request,
            bodySerializer = UpdateRecurringDebtRequest.serializer(),
        )
    }

    suspend fun deleteRecurringDebt(recurringDebtId: Long): Result<Unit> = apiResult {
        api.delete("recurring-debts/$recurringDebtId")
    }

    suspend fun toggleRecurringDebt(recurringDebtId: Long): Result<Boolean> = apiResult {
        api.post(
            path = "recurring-debts/$recurringDebtId/toggle",
            body = JsonObject(emptyMap()),
            bodySerializer = JsonObject.serializer(),
            serializer = ToggleRecurringResponse.serializer(),
        ).isActive
    }

    suspend fun generateRecurringDebt(recurringDebtId: Long): Result<Long> = apiResult {
        api.post(
            path = "recurring-debts/$recurringDebtId/generate",
            body = JsonObject(emptyMap()),
            bodySerializer = JsonObject.serializer(),
            serializer = GenerateRecurringResponse.serializer(),
        ).sharedDebtId
    }

    suspend fun listInstances(recurringDebtId: Long): Result<List<GeneratedInstanceDto>> = apiResult {
        api.get(
            "recurring-debts/$recurringDebtId/instances",
            ListSerializer(GeneratedInstanceDto.serializer()),
        )
    }

    suspend fun getRecurringDebtMembers(recurringDebtId: Long): Result<List<Long>> = apiResult {
        api.get(
            "recurring-debts/$recurringDebtId/members",
            ListSerializer(Long.serializer()),
        )
    }

    suspend fun getRecurringDebtShares(
        recurringDebtId: Long,
    ): Result<List<RecurringDebtMemberDto>> = apiResult {
        api.get(
            "recurring-debts/$recurringDebtId/shares",
            ListSerializer(RecurringDebtMemberDto.serializer()),
        )
    }
}
