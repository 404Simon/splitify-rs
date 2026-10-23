use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::maps::models::{
    DEFAULT_MARKER_EMOJI, MapConfig, MapMarker, PlaceSearchResult,
};

#[derive(Deserialize)]
pub struct MarkerRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default = "default_marker_emoji")]
    pub emoji: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize)]
pub struct CreatedResponse {
    pub id: i64,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

fn default_marker_emoji() -> String {
    DEFAULT_MARKER_EMOJI.to_string()
}

/// Group-scoped marker routes, nested under `/api/v1/groups`.
pub fn group_router() -> Router<crate::state::AppState> {
    Router::new().route("/{group_id}/markers", get(list_markers).post(create_marker))
}

/// Marker resource routes, nested under `/api/v1/markers`.
pub fn marker_router() -> Router<crate::state::AppState> {
    Router::new().route("/{marker_id}", patch(update_marker).delete(delete_marker))
}

/// Map config and geocoding routes, nested under `/api/v1/map`.
pub fn public_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/config", get(map_config))
        .route("/search", get(search_places))
}

async fn list_markers(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<MapMarker>>, ApiError> {
    let markers = crate::features::maps::service::list_markers(&pool, user.id, group_id).await?;
    Ok(Json(markers))
}

async fn create_marker(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<MarkerRequest>,
) -> Result<(StatusCode, Json<CreatedResponse>), ApiError> {
    let id = crate::features::maps::service::create_marker(
        &pool,
        user.id,
        group_id,
        payload.name,
        payload.description,
        payload.address,
        payload.emoji,
        payload.latitude,
        payload.longitude,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(CreatedResponse { id })))
}

async fn update_marker(
    Path(marker_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<MarkerRequest>,
) -> Result<StatusCode, ApiError> {
    crate::features::maps::service::update_marker(
        &pool,
        user.id,
        marker_id,
        payload.name,
        payload.description,
        payload.address,
        payload.emoji,
        payload.latitude,
        payload.longitude,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_marker(
    Path(marker_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::maps::service::delete_marker(&pool, user.id, marker_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn map_config(_user: ApiUser) -> Result<Json<MapConfig>, ApiError> {
    let config = crate::features::maps::service::map_config()?;
    Ok(Json(config))
}

async fn search_places(
    Query(query): Query<SearchQuery>,
    _user: ApiUser,
) -> Result<Json<Vec<PlaceSearchResult>>, ApiError> {
    let results = crate::features::maps::service::search_places(query.q).await?;
    Ok(Json(results))
}
