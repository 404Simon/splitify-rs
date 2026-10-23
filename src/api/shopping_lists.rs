use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::shopping_lists::EventBroadcaster;
use crate::features::shopping_lists::models::{
    ShoppingList, ShoppingListActivity, ShoppingListItem, ShoppingListSummary,
};

const ACTIVITY_LIMIT: i64 = 10;

#[derive(Deserialize)]
pub struct CreateShoppingListRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateShoppingListRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ItemRequest {
    pub name: String,
    pub quantity: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize)]
pub struct IdResponse {
    pub id: i64,
}

#[derive(Serialize)]
pub struct ToggleItemResponse {
    pub is_completed: bool,
}

/// Group-scoped shopping-list routes, nested under `/api/v1/groups`.
pub fn group_router() -> Router<crate::state::AppState> {
    Router::new().route(
        "/{group_id}/shopping-lists",
        get(list_lists).post(create_list),
    )
}

/// Shopping-list resource routes, nested under `/api/v1/shopping-lists`.
pub fn resource_router() -> Router<crate::state::AppState> {
    Router::new()
        .route(
            "/{list_id}",
            get(get_list).patch(update_list).delete(delete_list),
        )
        .route("/{list_id}/items", get(list_items).post(add_item))
        .route(
            "/{list_id}/items/{item_id}",
            patch(update_item).delete(delete_item),
        )
        .route("/{list_id}/items/{item_id}/toggle", post(toggle_item))
        .route("/{list_id}/activity", get(list_activity))
}

async fn list_lists(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<ShoppingListSummary>>, ApiError> {
    let lists =
        crate::features::shopping_lists::service::list_shopping_lists(&pool, user.id, group_id)
            .await?;
    Ok(Json(lists))
}

async fn create_list(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<CreateShoppingListRequest>,
) -> Result<(StatusCode, Json<IdResponse>), ApiError> {
    let id = crate::features::shopping_lists::service::create_shopping_list(
        &pool,
        user.id,
        group_id,
        payload.name,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

async fn get_list(
    Path(list_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<ShoppingList>, ApiError> {
    let list = crate::features::shopping_lists::service::get_shopping_list(&pool, user.id, list_id)
        .await?;
    Ok(Json(list))
}

async fn update_list(
    Path(list_id): Path<i64>,
    State(pool): State<SqlitePool>,
    State(broadcaster): State<EventBroadcaster>,
    user: ApiUser,
    Json(payload): Json<UpdateShoppingListRequest>,
) -> Result<StatusCode, ApiError> {
    crate::features::shopping_lists::service::update_shopping_list(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        payload.name,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_list(
    Path(list_id): Path<i64>,
    State(pool): State<SqlitePool>,
    State(broadcaster): State<EventBroadcaster>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::shopping_lists::service::delete_shopping_list(
        &pool,
        user.id,
        &broadcaster,
        list_id,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_items(
    Path(list_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<ShoppingListItem>>, ApiError> {
    let items =
        crate::features::shopping_lists::service::list_items(&pool, user.id, list_id).await?;
    Ok(Json(items))
}

async fn add_item(
    Path(list_id): Path<i64>,
    State(pool): State<SqlitePool>,
    State(broadcaster): State<EventBroadcaster>,
    user: ApiUser,
    Json(payload): Json<ItemRequest>,
) -> Result<(StatusCode, Json<IdResponse>), ApiError> {
    let id = crate::features::shopping_lists::service::add_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        payload.name,
        payload.quantity,
        payload.category,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

async fn toggle_item(
    Path((list_id, item_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
    State(broadcaster): State<EventBroadcaster>,
    user: ApiUser,
) -> Result<Json<ToggleItemResponse>, ApiError> {
    let is_completed = crate::features::shopping_lists::service::toggle_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        item_id,
    )
    .await?;

    Ok(Json(ToggleItemResponse { is_completed }))
}

async fn update_item(
    Path((list_id, item_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
    State(broadcaster): State<EventBroadcaster>,
    user: ApiUser,
    Json(payload): Json<ItemRequest>,
) -> Result<StatusCode, ApiError> {
    crate::features::shopping_lists::service::update_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        item_id,
        payload.name,
        payload.quantity,
        payload.category,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_item(
    Path((list_id, item_id)): Path<(i64, i64)>,
    State(pool): State<SqlitePool>,
    State(broadcaster): State<EventBroadcaster>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::shopping_lists::service::delete_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        item_id,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_activity(
    Path(list_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<ShoppingListActivity>>, ApiError> {
    let activity = crate::features::shopping_lists::service::list_activity(
        &pool,
        user.id,
        list_id,
        ACTIVITY_LIMIT,
    )
    .await?;

    Ok(Json(activity))
}
