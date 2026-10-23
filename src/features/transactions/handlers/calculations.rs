//! Balance calculations combining shared debts and transactions

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
use crate::features::transactions::models::UserBalance;

/// Calculate user debts for a group (combines shared debts and transactions)
#[server(CalculateUserDebts)]
pub async fn calculate_user_debts(group_id: i64) -> Result<Vec<UserBalance>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::transactions::service::group_balances(&pool, user.id, group_id).await
}
