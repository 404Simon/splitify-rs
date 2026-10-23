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
use crate::features::shopping_lists::events::EventBroadcaster;
use crate::features::shopping_lists::models::*;

#[server(GetShoppingLists)]
pub async fn get_shopping_lists(group_id: i64) -> Result<Vec<ShoppingListSummary>, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::shopping_lists::service::list_shopping_lists(&pool, user.id, group_id).await
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

    crate::features::shopping_lists::service::get_shopping_list(&pool, user.id, list_id).await
}

#[server(CreateShoppingList)]
pub async fn create_shopping_list(group_id: i64, name: String) -> Result<i64, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::shopping_lists::service::create_shopping_list(&pool, user.id, group_id, name)
        .await
}

#[server(UpdateShoppingList)]
pub async fn update_shopping_list(list_id: i64, name: String) -> Result<(), ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    crate::features::shopping_lists::service::update_shopping_list(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        name,
    )
    .await
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

    crate::features::shopping_lists::service::delete_shopping_list(
        &pool,
        user.id,
        &broadcaster,
        list_id,
    )
    .await
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

    crate::features::shopping_lists::service::list_items(&pool, user.id, list_id).await
}

#[server(AddShoppingListItem)]
pub async fn add_shopping_list_item(
    list_id: i64,
    name: String,
    quantity: Option<String>,
    category: Option<String>,
) -> Result<i64, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    crate::features::shopping_lists::service::add_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        name,
        quantity,
        category,
    )
    .await
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

    let list_id = item_list_id(&pool, item_id).await?;

    crate::features::shopping_lists::service::toggle_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        item_id,
    )
    .await
}

#[server(UpdateShoppingListItem)]
pub async fn update_shopping_list_item(
    item_id: i64,
    name: String,
    quantity: Option<String>,
    category: Option<String>,
) -> Result<(), ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();
    let broadcaster = expect_context::<EventBroadcaster>();

    let list_id = item_list_id(&pool, item_id).await?;

    crate::features::shopping_lists::service::update_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        item_id,
        name,
        quantity,
        category,
    )
    .await
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

    let list_id = item_list_id(&pool, item_id).await?;

    crate::features::shopping_lists::service::delete_item(
        &pool,
        user.id,
        &broadcaster,
        list_id,
        item_id,
    )
    .await
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

    crate::features::shopping_lists::service::list_activity(&pool, user.id, list_id, limit).await
}

#[cfg(feature = "ssr")]
async fn item_list_id(pool: &SqlitePool, item_id: i64) -> Result<i64, ServerFnError> {
    sqlx::query_scalar!(
        "SELECT shopping_list_id FROM shopping_list_items WHERE id = ?",
        item_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Item not found"))
}
