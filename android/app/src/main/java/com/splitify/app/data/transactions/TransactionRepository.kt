package com.splitify.app.data.transactions

import com.splitify.app.data.dto.CreateTransactionRequest
import com.splitify.app.data.dto.IdResponse
import com.splitify.app.data.dto.TransactionDto
import com.splitify.app.data.dto.UserBalanceDto
import com.splitify.app.data.network.ApiClient
import com.splitify.app.data.network.apiResult
import kotlinx.serialization.builtins.ListSerializer

class TransactionRepository(private val api: ApiClient) {

    suspend fun listTransactions(groupId: Long): Result<List<TransactionDto>> = apiResult {
        api.get("groups/$groupId/transactions", ListSerializer(TransactionDto.serializer()))
    }

    suspend fun createTransaction(
        groupId: Long,
        recipientId: Long,
        amount: String,
        description: String?,
    ): Result<Long> = apiResult {
        api.post(
            path = "groups/$groupId/transactions",
            body = CreateTransactionRequest(
                recipientId = recipientId,
                amount = amount,
                description = description,
            ),
            bodySerializer = CreateTransactionRequest.serializer(),
            serializer = IdResponse.serializer(),
        ).id
    }

    suspend fun updateTransaction(
        transactionId: Long,
        recipientId: Long,
        amount: String,
        description: String?,
    ): Result<Unit> = apiResult {
        api.patch(
            path = "transactions/$transactionId",
            body = CreateTransactionRequest(
                recipientId = recipientId,
                amount = amount,
                description = description,
            ),
            bodySerializer = CreateTransactionRequest.serializer(),
        )
    }

    suspend fun deleteTransaction(transactionId: Long): Result<Unit> = apiResult {
        api.delete("transactions/$transactionId")
    }

    suspend fun getBalances(groupId: Long): Result<List<UserBalanceDto>> = apiResult {
        api.get("groups/$groupId/balances", ListSerializer(UserBalanceDto.serializer()))
    }
}
