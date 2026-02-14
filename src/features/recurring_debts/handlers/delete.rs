//! Delete operations for recurring debts

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Server function: Delete a recurring debt
#[server(DeleteRecurringDebt)]
pub async fn delete_recurring_debt(recurring_debt_id: i64) -> Result<(), ServerFnError> {
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
        SELECT rd.id, rd.created_by
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
            "Only the creator can delete this recurring debt",
        ));
    }

    // Delete the recurring debt (cascade will handle pivot table and set NULL on shared_debts)
    sqlx::query!(
        "DELETE FROM recurring_debts WHERE id = ?",
        recurring_debt_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
