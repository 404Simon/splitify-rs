use leptos::prelude::*;

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Server function: Update a shared debt
#[server(UpdateSharedDebt)]
pub async fn update_shared_debt(
    debt_id: i64,
    name: String,
    amount: String,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    // Validation
    if name.trim().is_empty() {
        return Err(ServerFnError::new("Debt name is required"));
    }

    if name.len() > 255 {
        return Err(ServerFnError::new(
            "Debt name must be 255 characters or less",
        ));
    }

    let amount_decimal = amount
        .parse::<Decimal>()
        .map_err(|_| ServerFnError::new("Invalid amount format"))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than zero"));
    }

    if member_ids.is_empty() {
        return Err(ServerFnError::new(
            "At least one member must be selected to split the debt",
        ));
    }

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Check if user is the creator of the debt
    let debt = sqlx::query!(
        "SELECT created_by, group_id FROM shared_debts WHERE id = ?",
        debt_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Shared debt not found"))?;

    if debt.created_by != user.id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the creator can update this debt",
        ));
    }

    // Validate all selected members are part of the group
    for member_id in &member_ids {
        let is_group_member = sqlx::query!(
            "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
            debt.group_id,
            member_id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        if is_group_member.count == 0 {
            return Err(ServerFnError::new(
                "Some selected members are not part of this group",
            ));
        }
    }

    // Start a transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Update the shared debt
    let amount_str = amount_decimal.to_string();
    sqlx::query!(
        "UPDATE shared_debts SET name = ?, amount = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        name,
        amount_str,
        debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Remove all existing members
    sqlx::query!(
        "DELETE FROM shared_debt_user WHERE shared_debt_id = ?",
        debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Add new members
    for member_id in member_ids {
        sqlx::query!(
            "INSERT INTO shared_debt_user (shared_debt_id, user_id) VALUES (?, ?)",
            debt_id,
            member_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
