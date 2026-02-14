//! Query operations for transactions (read-only operations)

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
use crate::features::transactions::models::{Transaction, TransactionWithDetails};

/// Get a single transaction by ID
#[server(GetTransaction)]
pub async fn get_transaction(
    group_id: i64,
    transaction_id: i64,
) -> Result<Transaction, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

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

    // Fetch transaction
    let record = sqlx::query!(
        r#"
        SELECT 
            id as "id!",
            group_id as "group_id!",
            payer_id as "payer_id!",
            recipient_id as "recipient_id!",
            amount,
            description,
            created_at,
            updated_at
        FROM transactions
        WHERE id = ? AND group_id = ?
        "#,
        transaction_id,
        group_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found".to_string()))?;

    Ok(Transaction {
        id: record.id,
        group_id: record.group_id,
        payer_id: record.payer_id,
        recipient_id: record.recipient_id,
        amount: record.amount,
        description: record.description,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

/// Get all transactions for a group
#[server(GetGroupTransactions)]
pub async fn get_group_transactions(
    group_id: i64,
) -> Result<Vec<TransactionWithDetails>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

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

    // Fetch transactions with payer and recipient usernames
    let records = sqlx::query!(
        r#"
        SELECT 
            t.id as "id!",
            t.group_id as "group_id!",
            t.payer_id as "payer_id!",
            payer.username as payer_username,
            t.recipient_id as "recipient_id!",
            recipient.username as recipient_username,
            t.amount,
            t.description,
            t.created_at,
            t.updated_at
        FROM transactions t
        JOIN users payer ON t.payer_id = payer.id
        JOIN users recipient ON t.recipient_id = recipient.id
        WHERE t.group_id = ?
        ORDER BY t.created_at DESC
        "#,
        group_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(records
        .into_iter()
        .map(|r| TransactionWithDetails {
            id: r.id,
            group_id: r.group_id,
            payer_id: r.payer_id,
            payer_username: r.payer_username,
            recipient_id: r.recipient_id,
            recipient_username: r.recipient_username,
            amount: r.amount,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}
