use leptos::prelude::*;

use super::models::{SharedDebtWithDetails, UserShare};

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use super::utils::calculate_shares;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Server function: Get a specific shared debt
#[server(GetSharedDebt)]
pub async fn get_shared_debt(debt_id: i64) -> Result<SharedDebtWithDetails, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Get the shared debt and verify user has access
    let debt = sqlx::query!(
        r#"
        SELECT 
            sd.id as "id!",
            sd.group_id as "group_id!",
            sd.created_by as "created_by!",
            sd.name,
            sd.amount,
            sd.created_at,
            sd.updated_at,
            u.username as creator_username,
            CASE WHEN sd.created_by = ? THEN 1 ELSE 0 END as "is_creator!: bool"
        FROM shared_debts sd
        INNER JOIN users u ON sd.created_by = u.id
        INNER JOIN group_members gm ON sd.group_id = gm.group_id
        WHERE sd.id = ? AND gm.user_id = ?
        "#,
        user.id,
        debt_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Shared debt not found or access denied"))?;

    let amount = debt
        .amount
        .parse::<Decimal>()
        .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

    Ok(SharedDebtWithDetails {
        id: debt.id,
        group_id: debt.group_id,
        created_by: debt.created_by,
        creator_username: debt.creator_username,
        name: debt.name,
        amount,
        created_at: debt.created_at,
        updated_at: debt.updated_at,
        is_creator: debt.is_creator,
    })
}

/// Server function: Get all shared debts for a group
#[server(GetGroupSharedDebts)]
pub async fn get_group_shared_debts(
    group_id: i64,
) -> Result<Vec<SharedDebtWithDetails>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

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
            "Unauthorized: Not a member of this group",
        ));
    }

    // Fetch all shared debts for this group with creator info
    let debts = sqlx::query!(
        r#"
        SELECT 
            sd.id as "id!",
            sd.group_id as "group_id!",
            sd.created_by as "created_by!",
            sd.name,
            sd.amount,
            sd.created_at,
            sd.updated_at,
            u.username as creator_username,
            CASE WHEN sd.created_by = ? THEN 1 ELSE 0 END as "is_creator!: bool"
        FROM shared_debts sd
        INNER JOIN users u ON sd.created_by = u.id
        WHERE sd.group_id = ?
        ORDER BY sd.created_at DESC
        "#,
        user.id,
        group_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let debts_with_details = debts
        .into_iter()
        .map(|row| {
            let amount = row
                .amount
                .parse::<Decimal>()
                .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

            Ok(SharedDebtWithDetails {
                id: row.id,
                group_id: row.group_id,
                created_by: row.created_by,
                creator_username: row.creator_username,
                name: row.name,
                amount,
                created_at: row.created_at,
                updated_at: row.updated_at,
                is_creator: row.is_creator,
            })
        })
        .collect::<Result<Vec<_>, ServerFnError>>()?;

    Ok(debts_with_details)
}

/// Server function: Get user IDs involved in a shared debt
#[server(GetSharedDebtMembers)]
pub async fn get_shared_debt_members(debt_id: i64) -> Result<Vec<i64>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Verify user has access to this debt
    let _debt = sqlx::query!(
        r#"
        SELECT sd.id as "id!", sd.group_id as "group_id!", sd.amount
        FROM shared_debts sd
        INNER JOIN group_members gm ON sd.group_id = gm.group_id
        WHERE sd.id = ? AND gm.user_id = ?
        "#,
        debt_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Shared debt not found or access denied"))?;

    // Get member IDs
    let members = sqlx::query!(
        "SELECT user_id FROM shared_debt_user WHERE shared_debt_id = ?",
        debt_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(members.into_iter().map(|row| row.user_id).collect())
}

/// Server function: Get user shares for a specific shared debt
#[server(GetSharedDebtShares)]
pub async fn get_shared_debt_shares(debt_id: i64) -> Result<Vec<UserShare>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Get the shared debt and verify user has access
    let debt = sqlx::query!(
        r#"
        SELECT sd.id as "id!", sd.group_id as "group_id!", sd.amount
        FROM shared_debts sd
        INNER JOIN group_members gm ON sd.group_id = gm.group_id
        WHERE sd.id = ? AND gm.user_id = ?
        "#,
        debt_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Shared debt not found or access denied"))?;

    let amount = debt
        .amount
        .parse::<Decimal>()
        .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

    // Get all users involved in this debt
    let users = sqlx::query!(
        r#"
        SELECT u.id as "id!", u.username
        FROM users u
        INNER JOIN shared_debt_user sdu ON u.id = sdu.user_id
        WHERE sdu.shared_debt_id = ?
        ORDER BY u.username
        "#,
        debt_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_data: Vec<(i64, String)> = users
        .into_iter()
        .map(|row| (row.id, row.username))
        .collect();

    let shares = calculate_shares(amount, &user_data);

    Ok(shares)
}

/// Server function: Create a new shared debt
#[server(CreateSharedDebt)]
pub async fn create_shared_debt(
    group_id: i64,
    name: String,
    amount: String,
    member_ids: Vec<i64>,
) -> Result<i64, ServerFnError> {
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
            "Unauthorized: Not a member of this group",
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

/// Server function: Delete a shared debt
#[server(DeleteSharedDebt)]
pub async fn delete_shared_debt(debt_id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Check if user is the creator of the debt
    let debt = sqlx::query!("SELECT created_by FROM shared_debts WHERE id = ?", debt_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Shared debt not found"))?;

    if debt.created_by != user.id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the creator can delete this debt",
        ));
    }

    // Delete the debt (CASCADE will handle related data)
    sqlx::query!("DELETE FROM shared_debts WHERE id = ?", debt_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
