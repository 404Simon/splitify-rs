use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

#[cfg(feature = "ssr")]
pub async fn verify_list_access(
    pool: &SqlitePool,
    user_id: i64,
    list_id: i64,
) -> Result<(), ServerFnError> {
    let is_member = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM shopping_lists sl
            INNER JOIN group_members gm ON sl.group_id = gm.group_id
            WHERE sl.id = ? AND gm.user_id = ?
        ) as "exists!"
        "#,
        list_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Not a member of this list's group".to_string(),
        ));
    }

    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn verify_list_creator(
    pool: &SqlitePool,
    user_id: i64,
    list_id: i64,
) -> Result<(), ServerFnError> {
    let is_creator = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM shopping_lists 
            WHERE id = ? AND created_by = ?
        ) as "exists!"
        "#,
        list_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_creator == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Only the creator can perform this action".to_string(),
        ));
    }

    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn verify_group_membership(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<(), ServerFnError> {
    let is_member = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM group_members 
            WHERE group_id = ? AND user_id = ?
        ) as "exists!"
        "#,
        group_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Not a member of this group".to_string(),
        ));
    }

    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn log_activity(
    pool: &SqlitePool,
    shopping_list_id: i64,
    user_id: i64,
    action: &str,
    item_name: &str,
) -> Result<(), ServerFnError> {
    sqlx::query!(
        r#"
        INSERT INTO shopping_list_activity 
        (shopping_list_id, user_id, action, item_name)
        VALUES (?, ?, ?, ?)
        "#,
        shopping_list_id,
        user_id,
        action,
        item_name
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), ServerFnError> {
    if name.trim().is_empty() {
        return Err(ServerFnError::new("Name cannot be empty".to_string()));
    }
    if name.len() > 255 {
        return Err(ServerFnError::new(
            "Name must be 255 characters or less".to_string(),
        ));
    }
    Ok(())
}
