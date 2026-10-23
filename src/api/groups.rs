use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::groups::models::{Group, GroupMemberInfo, GroupWithMembers};

#[derive(Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateGroupResponse {
    pub id: i64,
}

#[derive(Deserialize)]
pub struct UpdateGroupRequest {
    pub name: String,
    pub member_ids: Vec<i64>,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route(
            "/{group_id}",
            get(get_group).patch(update_group).delete(delete_group),
        )
        .route("/{group_id}/members", get(list_members))
        .route("/{group_id}/snapshot", get(super::sync::group_snapshot))
}

async fn list_groups(
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<GroupWithMembers>>, ApiError> {
    let groups = crate::features::groups::service::list_user_groups(&pool, user.id).await?;
    Ok(Json(groups))
}

async fn create_group(
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<CreateGroupResponse>), ApiError> {
    let id = crate::features::groups::service::create_group(&pool, user.id, payload.name).await?;

    Ok((StatusCode::CREATED, Json(CreateGroupResponse { id })))
}

async fn get_group(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Group>, ApiError> {
    let group = crate::features::groups::service::get_group(&pool, user.id, group_id).await?;
    Ok(Json(group))
}

async fn update_group(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<StatusCode, ApiError> {
    crate::features::groups::service::update_group(
        &pool,
        user.id,
        group_id,
        payload.name,
        payload.member_ids,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_group(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    crate::features::groups::service::delete_group(&pool, user.id, group_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_members(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<GroupMemberInfo>>, ApiError> {
    let members =
        crate::features::groups::service::list_group_members(&pool, user.id, group_id).await?;
    Ok(Json(members))
}
