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

    let details =
        crate::features::transactions::service::get_transaction(&pool, user.id, transaction_id)
            .await?;

    if details.group_id != group_id {
        return Err(ServerFnError::new("Transaction not found".to_string()));
    }

    Ok(Transaction {
        id: details.id,
        group_id: details.group_id,
        payer_id: details.payer_id,
        recipient_id: details.recipient_id,
        amount: details.amount,
        description: details.description,
        created_at: details.created_at,
        updated_at: details.updated_at,
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

    crate::features::transactions::service::list_group_transactions(&pool, user.id, group_id).await
}
