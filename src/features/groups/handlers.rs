use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use super::models::{Group, GroupMemberInfo, GroupWithMembers};
use crate::features::auth::models::UserSession;
#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Get all groups for the current user
#[server(GetUserGroups)]
pub async fn get_user_groups() -> Result<Vec<GroupWithMembers>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    super::service::list_user_groups(&pool, user.id).await
}

/// Server function: Get a specific group with details
#[server(GetGroup)]
pub async fn get_group(group_id: i64) -> Result<Group, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    super::service::get_group(&pool, user.id, group_id).await
}

/// Server function: Get members of a group
#[server(GetGroupMembers)]
pub async fn get_group_members(group_id: i64) -> Result<Vec<GroupMemberInfo>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    super::service::list_group_members(&pool, user.id, group_id).await
}

/// Server function: Create a new group
#[server(CreateGroup)]
pub async fn create_group(name: String) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    super::service::create_group(&pool, user.id, name).await
}

/// Server function: Update a group
#[server(UpdateGroup)]
pub async fn update_group(
    group_id: i64,
    name: String,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    super::service::update_group(&pool, user.id, group_id, name, member_ids).await
}

/// Server function: Delete a group
#[server(DeleteGroup)]
pub async fn delete_group(group_id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    super::service::delete_group(&pool, user.id, group_id).await
}

/// Server function: Get all users for member selection
#[server(GetAllUsers)]
pub async fn get_all_users() -> Result<Vec<UserSession>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    super::service::list_other_users(&pool, user.id).await
}
