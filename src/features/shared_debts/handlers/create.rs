use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
#[cfg(feature = "ssr")]
use crate::validation::{validate_amount, validate_name};

/// Server function: Create a new shared debt
#[server(CreateSharedDebt)]
pub async fn create_shared_debt(
    group_id: i64,
    name: String,
    amount: String,
    member_ids: Vec<i64>,
) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    // Validate debt name
    let name = validate_name(&name, 1, 255, "Debt name")?;

    // Validate amount
    let amount_decimal = validate_amount(&amount)?;

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
        .ok_or_else(|| ServerFnError::new("Not authenticated. Please log in."))?;

    let pool = expect_context::<SqlitePool>();

    // Check if user is a member of the group
    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member.count == 0 {
        return Err(ServerFnError::new(
            "You don't have permission to access this group",
        ));
    }

    // Validate all selected members are part of the group
    for member_id in &member_ids {
        let is_group_member = sqlx::query!(
            "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
            group_id,
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

    // Insert the shared debt
    let amount_str = amount_decimal.to_string();
    let result = sqlx::query!(
        "INSERT INTO shared_debts (group_id, created_by, name, amount) VALUES (?, ?, ?, ?)",
        group_id,
        user.id,
        name,
        amount_str
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let debt_id = result.last_insert_rowid();

    // Add members to the shared debt
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

    Ok(debt_id)
}
