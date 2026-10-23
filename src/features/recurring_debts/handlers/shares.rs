//! Share calculations for recurring debts

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
use crate::features::recurring_debts::models::RecurringDebtMember;

/// Server function: Get member shares for a recurring debt (for display)
#[server(GetRecurringDebtShares)]
pub async fn get_recurring_debt_shares(
    recurring_debt_id: i64,
) -> Result<Vec<RecurringDebtMember>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::list_shares(&pool, user.id, recurring_debt_id).await
}
