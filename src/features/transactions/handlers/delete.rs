//! Delete operations for transactions

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Delete a transaction
#[server(DeleteTransaction)]
pub async fn delete_transaction(group_id: i64, transaction_id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    let existing = sqlx::query!(
        "SELECT group_id FROM transactions WHERE id = ?",
        transaction_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if existing.is_none_or(|row| row.group_id != group_id) {
        return Err(ServerFnError::new("Transaction not found"));
    }

    crate::features::transactions::service::delete_transaction(&pool, user.id, transaction_id).await
}
