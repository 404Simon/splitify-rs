//! Share calculations for recurring debts

use leptos::prelude::*;

use crate::features::recurring_debts::models::RecurringDebtMember;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use crate::features::shared_debts::utils::calculate_shares;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

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

    // Verify user has access to this recurring debt and get amount
    let debt = sqlx::query!(
        r#"
        SELECT rd.amount
        FROM recurring_debts rd
        INNER JOIN group_members gm ON rd.group_id = gm.group_id
        WHERE rd.id = ? AND gm.user_id = ?
        "#,
        recurring_debt_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Not authorized"))?;

    let amount = debt
        .amount
        .parse::<Decimal>()
        .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

    // Get members with usernames
    let members = sqlx::query!(
        r#"
        SELECT u.id as "user_id!", u.username
        FROM recurring_debt_user rdu
        INNER JOIN users u ON rdu.user_id = u.id
        WHERE rdu.recurring_debt_id = ?
        ORDER BY u.username
        "#,
        recurring_debt_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_data: Vec<(i64, String)> = members
        .into_iter()
        .map(|m| (m.user_id, m.username))
        .collect();

    let shares = calculate_shares(amount, &user_data);

    Ok(shares
        .into_iter()
        .map(|s| RecurringDebtMember {
            user_id: s.user_id,
            username: s.username,
            share_amount: s.share_amount,
        })
        .collect())
}
