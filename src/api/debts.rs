use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::shared_debts::models::{SharedDebtWithDetails, UserShare};

#[derive(Deserialize)]
pub struct DebtPayload {
    pub name: String,
    pub amount: String,
    pub member_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct CreatedResponse {
    pub id: i64,
}

/// Group-scoped debt routes, nested under `/api/v1/groups`.
pub fn group_router() -> Router<crate::state::AppState> {
    Router::new().route("/{group_id}/debts", get(list_group_debts).post(create_debt))
}

/// Debt resource routes, nested under `/api/v1/debts`.
pub fn resource_router() -> Router<crate::state::AppState> {
    Router::new()
        .route(
            "/{debt_id}",
            get(get_debt).patch(update_debt).delete(delete_debt),
        )
        .route("/{debt_id}/members", get(list_debt_members))
        .route("/{debt_id}/shares", get(list_debt_shares))
}

async fn list_group_debts(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<SharedDebtWithDetails>>, ApiError> {
    let debts =
        crate::features::shared_debts::service::list_group_debts(&pool, user.id, group_id).await?;
    Ok(Json(debts))
}

async fn create_debt(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<DebtPayload>,
) -> Result<(StatusCode, Json<CreatedResponse>), ApiError> {
    let id = crate::features::shared_debts::service::create_debt(
        &pool,
        user.id,
        group_id,
        payload.name,
        payload.amount,
        payload.member_ids,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(CreatedResponse { id })))
}

async fn get_debt(
    Path(debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<SharedDebtWithDetails>, ApiError> {
    let debt = crate::features::shared_debts::service::get_debt(&pool, user.id, debt_id).await?;
    Ok(Json(debt))
}

async fn update_debt(
    Path(debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<DebtPayload>,
) -> Result<StatusCode, ApiError> {
    crate::features::shared_debts::service::update_debt(
        &pool,
        user.id,
        debt_id,
        payload.name,
        payload.amount,
        payload.member_ids,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_debt(
    Path(debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::shared_debts::service::delete_debt(&pool, user.id, debt_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_debt_members(
    Path(debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<i64>>, ApiError> {
    let members =
        crate::features::shared_debts::service::list_debt_members(&pool, user.id, debt_id).await?;
    Ok(Json(members))
}

async fn list_debt_shares(
    Path(debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<UserShare>>, ApiError> {
    let shares =
        crate::features::shared_debts::service::list_debt_shares(&pool, user.id, debt_id).await?;
    Ok(Json(shares))
}
