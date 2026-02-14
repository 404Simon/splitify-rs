//! Create operations for transactions

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Create a new transaction
#[server(CreateTransaction)]
pub async fn create_transaction(
    group_id: i64,
    recipient_id: i64,
    amount: String,
    description: Option<String>,
) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;
    use std::str::FromStr;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Validate amount
    let amount_decimal = Decimal::from_str(&amount)
        .map_err(|_| ServerFnError::new("Invalid amount format".to_string()))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new(
            "Amount must be greater than 0".to_string(),
        ));
    }

    // Check user is member of group
    let is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member == 0 {
        return Err(ServerFnError::new(
            "You are not a member of this group".to_string(),
        ));
    }

    // Check recipient is member of group
    let recipient_is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        recipient_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if recipient_is_member == 0 {
        return Err(ServerFnError::new(
            "Recipient is not a member of this group".to_string(),
        ));
    }

    // Don't allow self-transactions
    if user.id == recipient_id {
        return Err(ServerFnError::new(
            "Cannot create transaction with yourself".to_string(),
        ));
    }

    // Store amount rounded to 2 decimal places
    let amount_str = amount_decimal.round_dp(2).to_string();

    // Insert transaction
    let result = sqlx::query!(
        r#"
        INSERT INTO transactions (group_id, payer_id, recipient_id, amount, description)
        VALUES (?, ?, ?, ?, ?)
        "#,
        group_id,
        user.id,
        recipient_id,
        amount_str,
        description
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
}
