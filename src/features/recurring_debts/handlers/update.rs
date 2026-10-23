//! Update operations for recurring debts

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Update a recurring debt
#[server(UpdateRecurringDebt)]
pub async fn update_recurring_debt(
    recurring_debt_id: i64,
    name: String,
    amount: String,
    frequency: String,
    end_date: Option<String>,
    is_active: bool,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::update(
        &pool,
        user.id,
        recurring_debt_id,
        name,
        amount,
        frequency,
        end_date,
        is_active,
        member_ids,
    )
    .await
}
