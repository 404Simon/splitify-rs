use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::transactions::models::{TransactionWithDetails, UserBalance};

#[derive(Deserialize)]
pub struct CreateTransactionPayload {
    pub recipient_id: i64,
    pub amount: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTransactionPayload {
    pub recipient_id: i64,
    pub amount: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct CreatedResponse {
    pub id: i64,
}

/// Group-scoped transaction/balance routes, nested under `/api/v1/groups`.
pub fn group_router() -> Router<crate::state::AppState> {
    Router::new()
        .route(
            "/{group_id}/transactions",
            get(list_group_transactions).post(create_transaction),
        )
        .route("/{group_id}/balances", get(group_balances))
}

/// Transaction resource routes, nested under `/api/v1/transactions`.
pub fn resource_router() -> Router<crate::state::AppState> {
    Router::new().route(
        "/{transaction_id}",
        get(get_transaction)
            .patch(update_transaction)
            .delete(delete_transaction),
    )
}

async fn list_group_transactions(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<TransactionWithDetails>>, ApiError> {
    let transactions =
        crate::features::transactions::service::list_group_transactions(&pool, user.id, group_id)
            .await?;
    Ok(Json(transactions))
}

async fn create_transaction(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<CreateTransactionPayload>,
) -> Result<(StatusCode, Json<CreatedResponse>), ApiError> {
    let id = crate::features::transactions::service::create_transaction(
        &pool,
        user.id,
        group_id,
        payload.recipient_id,
        payload.amount,
        payload.description,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(CreatedResponse { id })))
}

async fn get_transaction(
    Path(transaction_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<TransactionWithDetails>, ApiError> {
    let transaction =
        crate::features::transactions::service::get_transaction(&pool, user.id, transaction_id)
            .await?;
    Ok(Json(transaction))
}

async fn update_transaction(
    Path(transaction_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<UpdateTransactionPayload>,
) -> Result<StatusCode, ApiError> {
    crate::features::transactions::service::update_transaction(
        &pool,
        user.id,
        transaction_id,
        payload.recipient_id,
        payload.amount,
        payload.description,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_transaction(
    Path(transaction_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::transactions::service::delete_transaction(&pool, user.id, transaction_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn group_balances(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<UserBalance>>, ApiError> {
    let balances =
        crate::features::transactions::service::group_balances(&pool, user.id, group_id).await?;
    Ok(Json(balances))
}
