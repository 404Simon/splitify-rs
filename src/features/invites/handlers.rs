use leptos::prelude::*;

use super::models::{InviteListItem, InviteWithGroup};

#[cfg(feature = "ssr")]
use super::utils::{calculate_expiration, generate_invite_uuid, is_invite_valid};

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Server function: Get all invites for a group
#[server(GetGroupInvites)]
pub async fn get_group_invites(group_id: i64) -> Result<Vec<InviteListItem>, ServerFnError> {
    use sqlx::SqlitePool;
    use time::format_description;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Check if user is the group creator
    let group = sqlx::query!("SELECT created_by, name FROM groups WHERE id = ?", group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user.id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can view invites",
        ));
    }

    // Fetch all invites for the group
    let invites = sqlx::query!(
        r#"
        SELECT uuid, group_id, name, 
               CASE WHEN is_reusable = 1 THEN true ELSE false END as "is_reusable!: bool",
               duration_days, 
               created_at, 
               updated_at
        FROM invites 
        WHERE group_id = ?
        ORDER BY created_at DESC
        "#,
        group_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Format for date display
    let format = format_description::parse("[month repr:long] [day], [year]")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Convert to InviteListItem
    let invite_list = invites
        .into_iter()
        .map(|inv| {
            let expires_at = calculate_expiration(&inv.created_at, inv.duration_days);
            let expiration_date = expires_at
                .format(&format)
                .unwrap_or_else(|_| "Invalid date".to_string());

            InviteListItem {
                uuid: inv.uuid,
                name: inv.name,
                is_reusable: inv.is_reusable,
                expiration_date,
            }
        })
        .collect();

    Ok(invite_list)
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

    // Validate duration_days
    if !(1..=30).contains(&duration_days) {
        return Err(ServerFnError::new("Duration must be between 1 and 30 days"));
    }

    // Validate name length if provided
    if let Some(ref n) = name {
        if n.len() > 128 {
            return Err(ServerFnError::new("Name must be 128 characters or less"));
        }
    }

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Check if user is the group creator
    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user.id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can create invites",
        ));
    }

    // Generate UUID
    let uuid = generate_invite_uuid();

    // Insert invite
    let is_reusable_int = i32::from(is_reusable);
    sqlx::query!(
        "INSERT INTO invites (uuid, group_id, name, is_reusable, duration_days) VALUES (?, ?, ?, ?, ?)",
        uuid,
        group_id,
        name,
        is_reusable_int,
        duration_days
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(uuid)
}

/// Server function: Get invite by UUID (for public display)
#[server(GetInviteByUuid)]
pub async fn get_invite_by_uuid(uuid: String) -> Result<InviteWithGroup, ServerFnError> {
    use sqlx::SqlitePool;

    let pool = expect_context::<SqlitePool>();

    // Fetch invite with group information
    let invite = sqlx::query!(
        r#"
        SELECT 
            i.uuid, i.group_id, i.name, 
            CASE WHEN i.is_reusable = 1 THEN true ELSE false END as "is_reusable!: bool",
            i.duration_days, 
            i.created_at, 
            i.updated_at,
            g.name as group_name
        FROM invites i
        INNER JOIN groups g ON i.group_id = g.id
        WHERE i.uuid = ?
        "#,
        uuid
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Invite not found"))?;

    let is_valid = is_invite_valid(&invite.created_at, invite.duration_days);
    let expires_at = calculate_expiration(&invite.created_at, invite.duration_days);

    Ok(InviteWithGroup {
        uuid: invite.uuid,
        group_id: invite.group_id,
        group_name: invite.group_name,
        name: invite.name,
        is_reusable: invite.is_reusable,
        duration_days: invite.duration_days,
        created_at: invite.created_at,
        is_valid,
        expires_at,
    })
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

    // Start a transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Fetch invite
    let invite = sqlx::query!(
        r#"
        SELECT 
            uuid, group_id, 
            CASE WHEN is_reusable = 1 THEN true ELSE false END as "is_reusable!: bool",
            duration_days, created_at
        FROM invites 
        WHERE uuid = ?
        "#,
        uuid
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Invite not found"))?;

    // Check if invite is valid
    if !is_invite_valid(&invite.created_at, invite.duration_days) {
        return Err(ServerFnError::new("The invite is not valid or has expired"));
    }

    // Check if group exists
    let group = sqlx::query!("SELECT id FROM groups WHERE id = ?", invite.group_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("The group associated with this invite was not found"))?;

    let group_id = group.id;

    // Check if user is already a member
    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user.id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member.count > 0 {
        return Err(ServerFnError::new("You are already a member of this group"));
    }

    // Add user to group
    sqlx::query!(
        "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)",
        group_id,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Delete single-use invites
    if !invite.is_reusable {
        sqlx::query!("DELETE FROM invites WHERE uuid = ?", uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(group_id)
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

    // Check if user is the group creator
    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user.id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can delete invites",
        ));
    }

    // Delete the invite
    sqlx::query!(
        "DELETE FROM invites WHERE uuid = ? AND group_id = ?",
        uuid,
        group_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
