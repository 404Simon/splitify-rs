//! Invite business logic shared by the web server functions and the REST API.

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;
#[cfg(feature = "ssr")]
use time::format_description;

#[cfg(feature = "ssr")]
use super::models::{InviteListItem, InviteWithGroup};
#[cfg(feature = "ssr")]
use super::utils::{calculate_expiration, generate_invite_uuid, is_invite_valid};

/// List every invite for a group (group creator only).
#[cfg(feature = "ssr")]
pub async fn list_group_invites(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Vec<InviteListItem>, ServerFnError> {
    let group = sqlx::query!("SELECT created_by, name FROM groups WHERE id = ?", group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user_id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can view invites",
        ));
    }

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
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let format = format_description::parse_borrowed::<2>("[month repr:long] [day], [year]")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

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

/// Create an invite for a group (group creator only).
#[cfg(feature = "ssr")]
pub async fn create_invite(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
    name: Option<String>,
    is_reusable: bool,
    duration_days: i64,
) -> Result<String, ServerFnError> {
    if !(1..=30).contains(&duration_days) {
        return Err(ServerFnError::new("Duration must be between 1 and 30 days"));
    }

    if let Some(ref n) = name
        && n.len() > 128
    {
        return Err(ServerFnError::new("Name must be 128 characters or less"));
    }

    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user_id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can create invites",
        ));
    }

    let uuid = generate_invite_uuid();

    let is_reusable_int = i32::from(is_reusable);
    sqlx::query!(
        "INSERT INTO invites (uuid, group_id, name, is_reusable, duration_days) VALUES (?, ?, ?, ?, ?)",
        uuid,
        group_id,
        name,
        is_reusable_int,
        duration_days
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(uuid)
}

/// Fetch a public invite preview by UUID (no authentication required).
#[cfg(feature = "ssr")]
pub async fn get_invite_public(
    pool: &SqlitePool,
    uuid: &str,
) -> Result<InviteWithGroup, ServerFnError> {
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
    .fetch_optional(pool)
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

/// Accept an invite, adding the user to its group. Returns the group id.
#[cfg(feature = "ssr")]
pub async fn accept_invite(
    pool: &SqlitePool,
    user_id: i64,
    uuid: &str,
) -> Result<i64, ServerFnError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

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

    if !is_invite_valid(&invite.created_at, invite.duration_days) {
        return Err(ServerFnError::new("The invite is not valid or has expired"));
    }

    let group = sqlx::query!("SELECT id FROM groups WHERE id = ?", invite.group_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("The group associated with this invite was not found"))?;

    let group_id = group.id;

    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member.count > 0 {
        return Err(ServerFnError::new("You are already a member of this group"));
    }

    sqlx::query!(
        "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)",
        group_id,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !invite.is_reusable {
        sqlx::query!("DELETE FROM invites WHERE uuid = ?", uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(group_id)
}

/// Delete an invite from a group (group creator only).
#[cfg(feature = "ssr")]
pub async fn delete_invite(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
    uuid: &str,
) -> Result<(), ServerFnError> {
    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user_id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can delete invites",
        ));
    }

    sqlx::query!(
        "DELETE FROM invites WHERE uuid = ? AND group_id = ?",
        uuid,
        group_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
