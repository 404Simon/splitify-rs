use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
#[cfg(feature = "ssr")]
use crate::features::shopping_lists::events::*;
use crate::features::shopping_lists::models::*;
#[cfg(feature = "ssr")]
use crate::features::shopping_lists::utils::*;

#[server(GetShoppingLists)]
pub async fn get_shopping_lists(group_id: i64) -> Result<Vec<ShoppingListSummary>, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    verify_group_membership(&pool, user.id, group_id).await?;

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
    .fetch_all(&pool)
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

#[server(GetShoppingList)]
pub async fn get_shopping_list(list_id: i64) -> Result<ShoppingList, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    verify_list_access(&pool, user.id, list_id).await?;

    let list = sqlx::query_as!(
        ShoppingList,
        "SELECT * FROM shopping_lists WHERE id = ?",
        list_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(list)
}

#[server(CreateShoppingList)]
pub async fn create_shopping_list(group_id: i64, name: String) -> Result<i64, ServerFnError> {
    validate_name(&name)?;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    verify_group_membership(&pool, user.id, group_id).await?;

    let trimmed_name = name.trim();
    let result = sqlx::query!(
        "INSERT INTO shopping_lists (group_id, created_by, name) VALUES (?, ?, ?)",
        group_id,
        user.id,
        trimmed_name
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
}

#[server(UpdateShoppingList)]
pub async fn update_shopping_list(list_id: i64, name: String) -> Result<(), ServerFnError> {
    validate_name(&name)?;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    verify_list_access(&pool, user.id, list_id).await?;

    let trimmed_name = name.trim();
    let now = time::OffsetDateTime::now_utc();
    sqlx::query!(
        "UPDATE shopping_lists SET name = ?, updated_at = ? WHERE id = ?",
        trimmed_name,
        now,
        list_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(
        &broadcaster,
        list_id,
        ShoppingListEvent::ListUpdated {
            name: trimmed_name.to_string(),
        },
    );

    Ok(())
}

#[server(DeleteShoppingList)]
pub async fn delete_shopping_list(list_id: i64) -> Result<(), ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    verify_list_creator(&pool, user.id, list_id).await?;

    sqlx::query!("DELETE FROM shopping_lists WHERE id = ?", list_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(&broadcaster, list_id, ShoppingListEvent::ListDeleted);

    Ok(())
}

#[server(GetShoppingListItems)]
pub async fn get_shopping_list_items(list_id: i64) -> Result<Vec<ShoppingListItem>, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    verify_list_access(&pool, user.id, list_id).await?;

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
    .fetch_all(&pool)
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

#[server(AddShoppingListItem)]
pub async fn add_shopping_list_item(
    list_id: i64,
    name: String,
    quantity: Option<String>,
    category: Option<String>,
) -> Result<i64, ServerFnError> {
    validate_name(&name)?;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    verify_list_access(&pool, user.id, list_id).await?;

    let max_position = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(position), -1) FROM shopping_list_items WHERE shopping_list_id = ?",
        list_id
    )
    .fetch_one(&pool)
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
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let item_id = result.last_insert_rowid();

    broadcast_event(
        &broadcaster,
        list_id,
        ShoppingListEvent::ItemAdded {
            item_id,
            name: trimmed_name.to_string(),
            quantity: quantity.clone(),
            category: category.clone(),
            position,
            added_by_username: user.username.clone(),
        },
    );

    log_activity(&pool, list_id, user.id, "added_item", trimmed_name).await?;

    Ok(item_id)
}

#[server(ToggleShoppingListItem)]
pub async fn toggle_shopping_list_item(item_id: i64) -> Result<bool, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    let item = sqlx::query!(
        "SELECT shopping_list_id, is_completed, name FROM shopping_list_items WHERE id = ?",
        item_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    verify_list_access(&pool, user.id, item.shopping_list_id).await?;

    let new_completed = item.is_completed == 0;
    let now = time::OffsetDateTime::now_utc();
    let completed_by_value = if new_completed { Some(user.id) } else { None };
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
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(
        &broadcaster,
        item.shopping_list_id,
        ShoppingListEvent::ItemToggled {
            item_id,
            is_completed: new_completed,
            completed_by_username: if new_completed {
                Some(user.username.clone())
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
    log_activity(&pool, item.shopping_list_id, user.id, action, &item.name).await?;

    Ok(new_completed)
}

#[server(UpdateShoppingListItem)]
pub async fn update_shopping_list_item(
    item_id: i64,
    name: String,
    quantity: Option<String>,
    category: Option<String>,
) -> Result<(), ServerFnError> {
    validate_name(&name)?;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    let item = sqlx::query!(
        "SELECT shopping_list_id FROM shopping_list_items WHERE id = ?",
        item_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    verify_list_access(&pool, user.id, item.shopping_list_id).await?;

    let trimmed_name = name.trim();
    let now = time::OffsetDateTime::now_utc();
    sqlx::query!(
        r#"
        UPDATE shopping_list_items 
        SET name = ?, quantity = ?, category = ?, updated_at = ?
        WHERE id = ?
        "#,
        trimmed_name,
        quantity,
        category,
        now,
        item_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(
        &broadcaster,
        item.shopping_list_id,
        ShoppingListEvent::ItemUpdated {
            item_id,
            name: trimmed_name.to_string(),
            quantity: quantity.clone(),
            category: category.clone(),
        },
    );

    Ok(())
}

#[server(DeleteShoppingListItem)]
pub async fn delete_shopping_list_item(item_id: i64) -> Result<(), ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    let item = sqlx::query!(
        "SELECT shopping_list_id, name FROM shopping_list_items WHERE id = ?",
        item_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    verify_list_access(&pool, user.id, item.shopping_list_id).await?;

    sqlx::query!("DELETE FROM shopping_list_items WHERE id = ?", item_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    broadcast_event(
        &broadcaster,
        item.shopping_list_id,
        ShoppingListEvent::ItemDeleted { item_id },
    );

    log_activity(
        &pool,
        item.shopping_list_id,
        user.id,
        "deleted_item",
        &item.name,
    )
    .await?;

    Ok(())
}

#[server(GetShoppingListActivity)]
pub async fn get_shopping_list_activity(
    list_id: i64,
    limit: i64,
) -> Result<Vec<ShoppingListActivity>, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    verify_list_access(&pool, user.id, list_id).await?;

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
    .fetch_all(&pool)
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
