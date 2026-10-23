use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::invites::models::{InviteListItem, InviteWithGroup};

#[derive(Deserialize)]
pub struct CreateInviteRequest {
    pub name: Option<String>,
    pub is_reusable: bool,
    pub duration_days: i64,
}

#[derive(Serialize)]
pub struct CreateInviteResponse {
    pub uuid: String,
}

#[derive(Serialize)]
pub struct AcceptInviteResponse {
    pub group_id: i64,
}

/// Group-scoped invite routes, nested under `/api/v1/groups`.
pub fn group_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/{group_id}/invites", get(list_invites).post(create_invite))
        .route("/{group_id}/invites/{uuid}", delete(delete_invite))
}

/// Invite routes, nested under `/api/v1/invites`. The preview is public; accept
/// requires a Bearer token.
pub fn public_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/{uuid}", get(get_invite))
        .route("/{uuid}/accept", post(accept_invite))
}

async fn list_invites(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<InviteListItem>>, ApiError> {
    let invites =
        crate::features::invites::service::list_group_invites(&pool, user.id, group_id).await?;
    Ok(Json(invites))
}

async fn create_invite(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<CreateInviteResponse>), ApiError> {
    let uuid = crate::features::invites::service::create_invite(
        &pool,
        user.id,
        group_id,
        payload.name,
        payload.is_reusable,
        payload.duration_days,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(CreateInviteResponse { uuid })))
}

async fn delete_invite(
    Path((group_id, uuid)): Path<(i64, String)>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::invites::service::delete_invite(&pool, user.id, group_id, &uuid).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_invite(
    Path(uuid): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<InviteWithGroup>, ApiError> {
    let invite = crate::features::invites::service::get_invite_public(&pool, &uuid).await?;
    Ok(Json(invite))
}

async fn accept_invite(
    Path(uuid): Path<String>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<AcceptInviteResponse>, ApiError> {
    let group_id = crate::features::invites::service::accept_invite(&pool, user.id, &uuid).await?;
    Ok(Json(AcceptInviteResponse { group_id }))
}
