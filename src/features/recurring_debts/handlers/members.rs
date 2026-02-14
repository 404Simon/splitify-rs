//! Member-related queries for recurring debts

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Server function: Get members of a recurring debt
#[server(GetRecurringDebtMembers)]
pub async fn get_recurring_debt_members(recurring_debt_id: i64) -> Result<Vec<i64>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Verify user has access to this recurring debt
    let has_access = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM recurring_debts rd
        INNER JOIN group_members gm ON rd.group_id = gm.group_id
        WHERE rd.id = ? AND gm.user_id = ?
        "#,
        recurring_debt_id,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .count
        > 0;

    if !has_access {
        return Err(ServerFnError::new("Not authorized"));
    }

    // Get member IDs
    let members = sqlx::query!(
        r#"
        SELECT user_id as "user_id!"
        FROM recurring_debt_user
        WHERE recurring_debt_id = ?
        "#,
        recurring_debt_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(members.into_iter().map(|m| m.user_id).collect())
}
