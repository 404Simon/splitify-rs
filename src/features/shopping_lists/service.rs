//! Shopping list business logic shared by the web server functions and the REST API.

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

#[cfg(feature = "ssr")]
use super::events::{EventBroadcaster, broadcast_event};
#[cfg(feature = "ssr")]
use super::models::{
    ShoppingList, ShoppingListActivity, ShoppingListEvent, ShoppingListItem, ShoppingListSummary,
};
#[cfg(feature = "ssr")]
use super::utils::{
    log_activity, validate_name, verify_group_membership, verify_list_access, verify_list_creator,
};

/// List the shopping lists of a group the user belongs to.
#[cfg(feature = "ssr")]
pub async fn list_shopping_lists(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Vec<ShoppingListSummary>, ServerFnError> {
    verify_group_membership(pool, user_id, group_id).await?;

    let lists = sqlx::query!(
        r#"
        SELECT 
            sl.id as "id!",
            sl.name,
            sl.group_id as "group_id!",
            sl.created_by as "created_by!",
            u.username as creator_username,
            CAST(COUNT(sli.id) AS INTEGER) as "total_items!: i64",
            CAST(COUNT(CASE WHEN sli.is_completed = 1 THEN 1 END) AS INTEGER) as "completed_items!: i64"
        FROM shopping_lists sl
        INNER JOIN users u ON sl.created_by = u.id
        LEFT JOIN shopping_list_items sli ON sl.id = sli.shopping_list_id
        WHERE sl.group_id = ?
        GROUP BY sl.id
        ORDER BY sl.created_at DESC
        "#,
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(lists
        .into_iter()
        .map(|row| ShoppingListSummary {
            id: row.id,
            name: row.name,
            group_id: row.group_id,
            created_by: row.created_by,
            creator_username: row.creator_username,
            total_items: row.total_items,
            completed_items: row.completed_items,
        })
        .collect())
}

/// Fetch a single shopping list after verifying membership.
#[cfg(feature = "ssr")]
pub async fn get_shopping_list(
    pool: &SqlitePool,
    user_id: i64,
    list_id: i64,
) -> Result<ShoppingList, ServerFnError> {
    verify_list_access(pool, user_id, list_id).await?;

    sqlx::query_as!(
        ShoppingList,
        "SELECT * FROM shopping_lists WHERE id = ?",
        list_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Create a shopping list in a group the user belongs to.
#[cfg(feature = "ssr")]
pub async fn create_shopping_list(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
    name: String,
) -> Result<i64, ServerFnError> {
    validate_name(&name)?;

    verify_group_membership(pool, user_id, group_id).await?;

    let trimmed_name = name.trim();
    let result = sqlx::query!(
        "INSERT INTO shopping_lists (group_id, created_by, name) VALUES (?, ?, ?)",
        group_id,
        user_id,
        trimmed_name
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
}

/// Rename a shopping list and notify subscribers.
#[cfg(feature = "ssr")]
pub async fn update_shopping_list(
    pool: &SqlitePool,
    user_id: i64,
    broadcaster: &EventBroadcaster,
    list_id: i64,
    name: String,
) -> Result<(), ServerFnError> {
    validate_name(&name)?;

    verify_list_access(pool, user_id, list_id).await?;

    let trimmed_name = name.trim();
    let now = time::OffsetDateTime::now_utc();
    sqlx::query!(
        "UPDATE shopping_lists SET name = ?, updated_at = ? WHERE id = ?",
        trimmed_name,
        now,
        list_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(
        broadcaster,
        list_id,
        ShoppingListEvent::ListUpdated {
            name: trimmed_name.to_string(),
        },
    );

    Ok(())
}

/// Delete a shopping list (creator only) and notify subscribers.
#[cfg(feature = "ssr")]
pub async fn delete_shopping_list(
    pool: &SqlitePool,
    user_id: i64,
    broadcaster: &EventBroadcaster,
    list_id: i64,
) -> Result<(), ServerFnError> {
    verify_list_creator(pool, user_id, list_id).await?;

    sqlx::query!("DELETE FROM shopping_lists WHERE id = ?", list_id)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(broadcaster, list_id, ShoppingListEvent::ListDeleted);

    Ok(())
}

/// List the items of a shopping list after verifying membership.
#[cfg(feature = "ssr")]
pub async fn list_items(
    pool: &SqlitePool,
    user_id: i64,
    list_id: i64,
) -> Result<Vec<ShoppingListItem>, ServerFnError> {
    verify_list_access(pool, user_id, list_id).await?;

    let items = sqlx::query!(
        r#"
        SELECT 
            sli.id as "id!",
            sli.shopping_list_id as "shopping_list_id!",
            sli.name,
            sli.quantity,
            sli.category,
            sli.is_completed,
            sli.completed_by,
            sli.completed_at,
            sli.position as "position!",
            sli.created_at,
            sli.updated_at,
            u.username as "completed_by_username?"
        FROM shopping_list_items sli
        LEFT JOIN users u ON sli.completed_by = u.id
        WHERE sli.shopping_list_id = ?
        ORDER BY sli.position ASC, sli.created_at ASC
        "#,
        list_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(items
        .into_iter()
        .map(|row| ShoppingListItem {
            id: row.id,
            shopping_list_id: row.shopping_list_id,
            name: row.name,
            quantity: row.quantity,
            category: row.category,
            is_completed: row.is_completed != 0,
            completed_by: row.completed_by,
            completed_by_username: row.completed_by_username,
            completed_at: row.completed_at,
            position: row.position,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect())
}

/// Add an item to a shopping list and notify subscribers.
#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn add_item(
    pool: &SqlitePool,
    user_id: i64,
    broadcaster: &EventBroadcaster,
    list_id: i64,
    name: String,
    quantity: Option<String>,
    category: Option<String>,
) -> Result<i64, ServerFnError> {
    validate_name(&name)?;

    verify_list_access(pool, user_id, list_id).await?;

    let max_position = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(position), -1) FROM shopping_list_items WHERE shopping_list_id = ?",
        list_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let position = max_position + 1;

    let trimmed_name = name.trim();
    let result = sqlx::query!(
        r#"
        INSERT INTO shopping_list_items 
        (shopping_list_id, name, quantity, category, position) 
        VALUES (?, ?, ?, ?, ?)
        "#,
        list_id,
        trimmed_name,
        quantity,
        category,
        position
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let item_id = result.last_insert_rowid();

    broadcast_event(
        broadcaster,
        list_id,
        ShoppingListEvent::ItemAdded {
            item_id,
            name: trimmed_name.to_string(),
            quantity: quantity.clone(),
            category: category.clone(),
            position,
            added_by_username: username_for(pool, user_id).await?,
        },
    );

    log_activity(pool, list_id, user_id, "added_item", trimmed_name).await?;

    Ok(item_id)
}

/// Toggle an item's completion state and notify subscribers.
#[cfg(feature = "ssr")]
pub async fn toggle_item(
    pool: &SqlitePool,
    user_id: i64,
    broadcaster: &EventBroadcaster,
    list_id: i64,
    item_id: i64,
) -> Result<bool, ServerFnError> {
    verify_list_access(pool, user_id, list_id).await?;

    let item = sqlx::query!(
        "SELECT is_completed, name FROM shopping_list_items WHERE id = ? AND shopping_list_id = ?",
        item_id,
        list_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Item not found"))?;

    let new_completed = item.is_completed == 0;
    let now = time::OffsetDateTime::now_utc();
    let completed_by_value = if new_completed { Some(user_id) } else { None };
    let completed_at_value = if new_completed { Some(now) } else { None };

    sqlx::query!(
        r#"
        UPDATE shopping_list_items 
        SET is_completed = ?, 
            completed_by = ?, 
            completed_at = ?,
            updated_at = ?
        WHERE id = ?
        "#,
        new_completed,
        completed_by_value,
        completed_at_value,
        now,
        item_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(
        broadcaster,
        list_id,
        ShoppingListEvent::ItemToggled {
            item_id,
            is_completed: new_completed,
            completed_by_username: if new_completed {
                Some(username_for(pool, user_id).await?)
            } else {
                None
            },
        },
    );

    let action = if new_completed {
        "completed_item"
    } else {
        "uncompleted_item"
    };
    log_activity(pool, list_id, user_id, action, &item.name).await?;

    Ok(new_completed)
}

/// Update an item's fields and notify subscribers.
#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn update_item(
    pool: &SqlitePool,
    user_id: i64,
    broadcaster: &EventBroadcaster,
    list_id: i64,
    item_id: i64,
    name: String,
    quantity: Option<String>,
    category: Option<String>,
) -> Result<(), ServerFnError> {
    validate_name(&name)?;

    verify_list_access(pool, user_id, list_id).await?;

    let trimmed_name = name.trim();
    let now = time::OffsetDateTime::now_utc();
    let result = sqlx::query!(
        r#"
        UPDATE shopping_list_items 
        SET name = ?, quantity = ?, category = ?, updated_at = ?
        WHERE id = ? AND shopping_list_id = ?
        "#,
        trimmed_name,
        quantity,
        category,
        now,
        item_id,
        list_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Item not found"));
    }

    broadcast_event(
        broadcaster,
        list_id,
        ShoppingListEvent::ItemUpdated {
            item_id,
            name: trimmed_name.to_string(),
            quantity: quantity.clone(),
            category: category.clone(),
        },
    );

    Ok(())
}

/// Delete an item from a shopping list and notify subscribers.
#[cfg(feature = "ssr")]
pub async fn delete_item(
    pool: &SqlitePool,
    user_id: i64,
    broadcaster: &EventBroadcaster,
    list_id: i64,
    item_id: i64,
) -> Result<(), ServerFnError> {
    verify_list_access(pool, user_id, list_id).await?;

    let item = sqlx::query!(
        "SELECT name FROM shopping_list_items WHERE id = ? AND shopping_list_id = ?",
        item_id,
        list_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Item not found"))?;

    sqlx::query!(
        "DELETE FROM shopping_list_items WHERE id = ? AND shopping_list_id = ?",
        item_id,
        list_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(
        broadcaster,
        list_id,
        ShoppingListEvent::ItemDeleted { item_id },
    );

    log_activity(pool, list_id, user_id, "deleted_item", &item.name).await?;

    Ok(())
}

/// List recent activity for a shopping list after verifying membership.
#[cfg(feature = "ssr")]
pub async fn list_activity(
    pool: &SqlitePool,
    user_id: i64,
    list_id: i64,
    limit: i64,
) -> Result<Vec<ShoppingListActivity>, ServerFnError> {
    verify_list_access(pool, user_id, list_id).await?;

    let activities = sqlx::query!(
        r#"
        SELECT 
            sla.id as "id!",
            sla.shopping_list_id as "shopping_list_id!",
            sla.user_id as "user_id!",
            u.username,
            sla.action,
            sla.item_name,
            sla.created_at
        FROM shopping_list_activity sla
        INNER JOIN users u ON sla.user_id = u.id
        WHERE sla.shopping_list_id = ?
        ORDER BY sla.created_at DESC
        LIMIT ?
        "#,
        list_id,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(activities
        .into_iter()
        .map(|row| ShoppingListActivity {
            id: row.id,
            shopping_list_id: row.shopping_list_id,
            user_id: row.user_id,
            username: row.username,
            action: row.action,
            item_name: row.item_name,
            created_at: row.created_at,
        })
        .collect())
}

#[cfg(feature = "ssr")]
async fn username_for(pool: &SqlitePool, user_id: i64) -> Result<String, ServerFnError> {
    sqlx::query_scalar!("SELECT username FROM users WHERE id = ?", user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
