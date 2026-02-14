use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use super::models::{Group, GroupMemberInfo, GroupWithMembers};
use crate::features::auth::models::UserSession;
#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
#[cfg(feature = "ssr")]
use crate::validation::validate_name;

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

    // Fetch groups with member count
    let groups = sqlx::query!(
        r#"
        SELECT
            g.id as "id!",
            g.name,
            g.created_by,
            g.created_at,
            g.updated_at,
            COALESCE(COUNT(gm.user_id), 0) as "member_count!: i64",
            CASE WHEN g.created_by = ? THEN 1 ELSE 0 END as "is_admin!: bool"
        FROM groups g
        INNER JOIN group_members gm ON g.id = gm.group_id
        WHERE g.id IN (
            SELECT group_id FROM group_members WHERE user_id = ?
        )
        GROUP BY g.id
        ORDER BY g.updated_at DESC
        "#,
        user.id,
        user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let groups_with_members = groups
        .into_iter()
        .map(|row| GroupWithMembers {
            id: row.id,
            name: row.name,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
            member_count: row.member_count,
            is_admin: row.is_admin,
        })
        .collect();

    Ok(groups_with_members)
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

    // Check if user is a member of the group
    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member.count == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Not a member of this group",
        ));
    }

    // Fetch the group
    let group = sqlx::query!(
        "SELECT id, name, created_by, created_at, updated_at FROM groups WHERE id = ?",
        group_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Group not found"))?;

    Ok(Group {
        id: group.id,
        name: group.name,
        created_by: group.created_by,
        created_at: group.created_at,
        updated_at: group.updated_at,
    })
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

    // Check if user is a member of the group
    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member.count == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Not a member of this group",
        ));
    }

    // Get the group creator ID
    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    // Fetch all members with creator flag
    let members = sqlx::query!(
        r#"
        SELECT 
            u.id as "id!",
            u.username,
            CASE WHEN u.id = ? THEN 1 ELSE 0 END as "is_creator!: bool"
        FROM users u
        INNER JOIN group_members gm ON u.id = gm.user_id
        WHERE gm.group_id = ?
        ORDER BY (CASE WHEN u.id = ? THEN 0 ELSE 1 END), u.username ASC
        "#,
        group.created_by,
        group_id,
        group.created_by
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let member_infos = members
        .into_iter()
        .map(|row| GroupMemberInfo {
            id: row.id,
            username: row.username,
            is_creator: row.is_creator,
        })
        .collect();

    Ok(member_infos)
}

/// Server function: Create a new group
#[server(CreateGroup)]
pub async fn create_group(name: String) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    // Validate group name
    let name = validate_name(&name, 1, 255, "Group name")?;

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

    // Insert the group
    let result = sqlx::query!(
        "INSERT INTO groups (name, created_by) VALUES (?, ?)",
        name,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let group_id = result.last_insert_rowid();

    // Add creator as the first member
    sqlx::query!(
        "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)",
        group_id,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(group_id)
}

/// Server function: Update a group
#[server(UpdateGroup)]
pub async fn update_group(
    group_id: i64,
    name: String,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    // Validate group name
    let name = validate_name(&name, 1, 255, "Group name")?;

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
            "Unauthorized: Only the group creator can update the group",
        ));
    }

    // Prevent creator from removing themselves
    if !member_ids.contains(&user.id) {
        return Err(ServerFnError::new(
            "You cannot remove yourself from the group",
        ));
    }

    // Start a transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Update group name and updated_at timestamp
    sqlx::query!(
        "UPDATE groups SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        name,
        group_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Remove all existing members
    sqlx::query!("DELETE FROM group_members WHERE group_id = ?", group_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Add new members
    for member_id in member_ids {
        sqlx::query!(
            "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)",
            group_id,
            member_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
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

    // Check if user is the group creator
    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user.id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can delete the group",
        ));
    }

    // Delete the group (CASCADE will handle related data)
    sqlx::query!("DELETE FROM groups WHERE id = ?", group_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
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

    // Fetch all users except current user
    let users = sqlx::query_as::<_, UserSession>(
        "SELECT id, username FROM users WHERE id != ? ORDER BY username",
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(users)
}
