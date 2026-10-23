//! Toggle active status operations for recurring debts

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Toggle active status of a recurring debt
#[server(ToggleRecurringDebtActive)]
pub async fn toggle_recurring_debt_active(recurring_debt_id: i64) -> Result<bool, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::toggle_active(
        &pool,
        user.id,
        recurring_debt_id,
        None,
    )
    .await
}
