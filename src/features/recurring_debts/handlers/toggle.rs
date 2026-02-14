//! Toggle active status operations for recurring debts

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

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

    // Get the recurring debt and verify user is the creator
    let debt = sqlx::query!(
        r#"
        SELECT rd.id, rd.created_by, rd.is_active as "is_active!: bool"
        FROM recurring_debts rd
        WHERE rd.id = ?
        "#,
        recurring_debt_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Recurring debt not found"))?;

    if debt.created_by != user.id {
        return Err(ServerFnError::new(
            "Only the creator can toggle this recurring debt",
        ));
    }

    let new_status = !debt.is_active;

    // Toggle is_active
    sqlx::query!(
        "UPDATE recurring_debts SET is_active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        new_status,
        recurring_debt_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(new_status)
}
