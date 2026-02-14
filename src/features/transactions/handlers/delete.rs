//! Delete operations for transactions

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

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

    // Check transaction exists and user is the payer
    let existing = sqlx::query!(
        "SELECT payer_id FROM transactions WHERE id = ? AND group_id = ?",
        transaction_id,
        group_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found"))?;

    if existing.payer_id != user.id {
        return Err(ServerFnError::new(
            "You can only delete your own transactions",
        ));
    }

    // Delete transaction
    sqlx::query!("DELETE FROM transactions WHERE id = ?", transaction_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
