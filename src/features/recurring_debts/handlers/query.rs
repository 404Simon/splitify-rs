//! Query operations for recurring debts (read-only operations)

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
use crate::features::recurring_debts::models::RecurringDebtWithDetails;

/// Server function: Get all recurring debts for a group
#[server(GetRecurringDebts)]
pub async fn get_recurring_debts(
    group_id: i64,
) -> Result<Vec<RecurringDebtWithDetails>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::list_for_group(&pool, user.id, group_id).await
}

/// Server function: Get a specific recurring debt
#[server(GetRecurringDebt)]
pub async fn get_recurring_debt(
    recurring_debt_id: i64,
) -> Result<RecurringDebtWithDetails, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::get(&pool, user.id, recurring_debt_id).await
}
