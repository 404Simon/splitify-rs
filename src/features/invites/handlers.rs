use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use super::models::{InviteListItem, InviteWithGroup};
#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Get all invites for a group
#[server(GetGroupInvites)]
pub async fn get_group_invites(group_id: i64) -> Result<Vec<InviteListItem>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::invites::service::list_group_invites(&pool, user.id, group_id).await
}

/// Server function: Create a new invite
#[server(CreateInvite)]
pub async fn create_invite(
    group_id: i64,
    name: Option<String>,
    is_reusable: bool,
    duration_days: i64,
) -> Result<String, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::invites::service::create_invite(
        &pool,
        user.id,
        group_id,
        name,
        is_reusable,
        duration_days,
    )
    .await
}

/// Server function: Get invite by UUID (for public display)
#[server(GetInviteByUuid)]
pub async fn get_invite_by_uuid(uuid: String) -> Result<InviteWithGroup, ServerFnError> {
    use sqlx::SqlitePool;

    let pool = expect_context::<SqlitePool>();

    crate::features::invites::service::get_invite_public(&pool, &uuid).await
}

/// Server function: Accept an invite
#[server(AcceptInvite)]
pub async fn accept_invite(uuid: String) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::invites::service::accept_invite(&pool, user.id, &uuid).await
}

/// Server function: Delete an invite
#[server(DeleteInvite)]
pub async fn delete_invite(uuid: String, group_id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::invites::service::delete_invite(&pool, user.id, group_id, &uuid).await
}
