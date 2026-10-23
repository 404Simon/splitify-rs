//! Create operations for recurring debts

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Create a new recurring debt
#[server(CreateRecurringDebt)]
pub async fn create_recurring_debt(
    group_id: i64,
    name: String,
    amount: String,
    frequency: String,
    start_date: String,
    end_date: Option<String>,
    member_ids: Vec<i64>,
) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::create(
        &pool, user.id, group_id, name, amount, frequency, start_date, end_date, member_ids,
    )
    .await
}
