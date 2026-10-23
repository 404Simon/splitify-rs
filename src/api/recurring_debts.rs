use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::recurring_debts::models::{
    GeneratedInstance, RecurringDebtMember, RecurringDebtWithDetails,
};

#[derive(Deserialize)]
pub struct CreateRecurringDebtRequest {
    pub name: String,
    pub amount: String,
    pub frequency: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub member_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct UpdateRecurringDebtRequest {
    pub name: String,
    pub amount: String,
    pub frequency: String,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub member_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct CreatedResponse {
    pub id: i64,
}

#[derive(Deserialize)]
pub struct ToggleRecurringDebtRequest {
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct ToggleResponse {
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct GenerateResponse {
    pub shared_debt_id: i64,
}

/// Group-scoped recurring-debt routes, nested under `/api/v1/groups`.
pub fn group_router() -> Router<crate::state::AppState> {
    Router::new().route(
        "/{group_id}/recurring-debts",
        get(list_recurring_debts).post(create_recurring_debt),
    )
}

/// Recurring-debt resource routes, nested under `/api/v1/recurring-debts`.
pub fn resource_router() -> Router<crate::state::AppState> {
    Router::new()
        .route(
            "/{recurring_debt_id}",
            get(get_recurring_debt)
                .patch(update_recurring_debt)
                .delete(delete_recurring_debt),
        )
        .route("/{recurring_debt_id}/toggle", post(toggle_recurring_debt))
        .route("/{recurring_debt_id}/generate", post(generate_now))
        .route("/{recurring_debt_id}/instances", get(list_instances))
        .route("/{recurring_debt_id}/members", get(list_members))
        .route("/{recurring_debt_id}/shares", get(list_shares))
}

async fn list_recurring_debts(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<RecurringDebtWithDetails>>, ApiError> {
    let debts =
        crate::features::recurring_debts::service::list_for_group(&pool, user.id, group_id).await?;
    Ok(Json(debts))
}

async fn create_recurring_debt(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<CreateRecurringDebtRequest>,
) -> Result<(StatusCode, Json<CreatedResponse>), ApiError> {
    let id = crate::features::recurring_debts::service::create(
        &pool,
        user.id,
        group_id,
        payload.name,
        payload.amount,
        payload.frequency,
        payload.start_date,
        payload.end_date,
        payload.member_ids,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(CreatedResponse { id })))
}

async fn get_recurring_debt(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<RecurringDebtWithDetails>, ApiError> {
    let debt =
        crate::features::recurring_debts::service::get(&pool, user.id, recurring_debt_id).await?;
    Ok(Json(debt))
}

async fn update_recurring_debt(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<UpdateRecurringDebtRequest>,
) -> Result<StatusCode, ApiError> {
    crate::features::recurring_debts::service::update(
        &pool,
        user.id,
        recurring_debt_id,
        payload.name,
        payload.amount,
        payload.frequency,
        payload.end_date,
        payload.is_active,
        payload.member_ids,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_recurring_debt(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::recurring_debts::service::delete(&pool, user.id, recurring_debt_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn toggle_recurring_debt(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    body: Bytes,
) -> Result<Json<ToggleResponse>, ApiError> {
    let desired = if body.is_empty() {
        None
    } else {
        let payload: ToggleRecurringDebtRequest = serde_json::from_slice(&body)
            .map_err(|error| ApiError::bad_request(format!("Invalid request body: {error}")))?;
        Some(payload.is_active)
    };

    let is_active = crate::features::recurring_debts::service::toggle_active(
        &pool,
        user.id,
        recurring_debt_id,
        desired,
    )
    .await?;

    Ok(Json(ToggleResponse { is_active }))
}

async fn generate_now(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<GenerateResponse>, ApiError> {
    let shared_debt_id =
        crate::features::recurring_debts::service::generate_now(&pool, user.id, recurring_debt_id)
            .await?;
    Ok(Json(GenerateResponse { shared_debt_id }))
}

async fn list_instances(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<GeneratedInstance>>, ApiError> {
    let instances = crate::features::recurring_debts::service::list_instances(
        &pool,
        user.id,
        recurring_debt_id,
    )
    .await?;
    Ok(Json(instances))
}

async fn list_members(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<i64>>, ApiError> {
    let members =
        crate::features::recurring_debts::service::list_members(&pool, user.id, recurring_debt_id)
            .await?;
    Ok(Json(members))
}

async fn list_shares(
    Path(recurring_debt_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<RecurringDebtMember>>, ApiError> {
    let shares =
        crate::features::recurring_debts::service::list_shares(&pool, user.id, recurring_debt_id)
            .await?;
    Ok(Json(shares))
}
