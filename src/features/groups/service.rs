#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

#[cfg(feature = "ssr")]
use super::models::{Group, GroupMemberInfo, GroupWithMembers};
#[cfg(feature = "ssr")]
use super::permissions::verify_group_membership;
#[cfg(feature = "ssr")]
use crate::features::auth::models::UserSession;
#[cfg(feature = "ssr")]
use crate::validation::validate_name;

/// Fetch all groups the user belongs to.
#[cfg(feature = "ssr")]
pub async fn list_user_groups(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<GroupWithMembers>, ServerFnError> {
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
        user_id,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(groups
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
        .collect())
}

/// Fetch a single group after verifying membership.
#[cfg(feature = "ssr")]
pub async fn get_group(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Group, ServerFnError> {
    verify_group_membership(pool, user_id, group_id).await?;

    let group = sqlx::query!(
        "SELECT id, name, created_by, created_at, updated_at FROM groups WHERE id = ?",
        group_id
    )
    .fetch_optional(pool)
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

/// List the members of a group after verifying membership.
#[cfg(feature = "ssr")]
pub async fn list_group_members(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Vec<GroupMemberInfo>, ServerFnError> {
    verify_group_membership(pool, user_id, group_id).await?;

    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

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
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(members
        .into_iter()
        .map(|row| GroupMemberInfo {
            id: row.id,
            username: row.username,
            is_creator: row.is_creator,
        })
        .collect())
}

/// Create a group and add the creator as its first member.
#[cfg(feature = "ssr")]
pub async fn create_group(
    pool: &SqlitePool,
    user_id: i64,
    name: String,
) -> Result<i64, ServerFnError> {
    let name = validate_name(&name, 1, 255, "Group name")?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let result = sqlx::query!(
        "INSERT INTO groups (name, created_by) VALUES (?, ?)",
        name,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let group_id = result.last_insert_rowid();

    sqlx::query!(
        "INSERT INTO group_members (group_id, user_id) VALUES (?, ?)",
        group_id,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(group_id)
}

/// Update a group's name and membership (creator only).
#[cfg(feature = "ssr")]
pub async fn update_group(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
    name: String,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    let name = validate_name(&name, 1, 255, "Group name")?;

    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user_id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can update the group",
        ));
    }

    if !member_ids.contains(&user_id) {
        return Err(ServerFnError::new(
            "You cannot remove yourself from the group",
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query!(
        "UPDATE groups SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        name,
        group_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query!("DELETE FROM group_members WHERE group_id = ?", group_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

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

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Delete a group (creator only).
#[cfg(feature = "ssr")]
pub async fn delete_group(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<(), ServerFnError> {
    let group = sqlx::query!("SELECT created_by FROM groups WHERE id = ?", group_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Group not found"))?;

    if group.created_by != user_id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the group creator can delete the group",
        ));
    }

    sqlx::query!("DELETE FROM groups WHERE id = ?", group_id)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// List every user except the current user.
#[cfg(feature = "ssr")]
pub async fn list_other_users(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<UserSession>, ServerFnError> {
    sqlx::query_as::<_, UserSession>(
        "SELECT id, username FROM users WHERE id != ? ORDER BY username",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}
