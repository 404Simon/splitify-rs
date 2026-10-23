//! Shared debt business logic shared by the web server functions and the REST API.

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use rust_decimal::Decimal;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

#[cfg(feature = "ssr")]
use super::models::{SharedDebtWithDetails, UserShare};
#[cfg(feature = "ssr")]
use super::utils::calculate_shares;
#[cfg(feature = "ssr")]
use crate::validation::{validate_amount, validate_name};

/// Fetch a single shared debt, verifying the user can access its group.
#[cfg(feature = "ssr")]
pub async fn get_debt(
    pool: &SqlitePool,
    user_id: i64,
    debt_id: i64,
) -> Result<SharedDebtWithDetails, ServerFnError> {
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
        user_id,
        debt_id,
        user_id
    )
    .fetch_optional(pool)
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

/// Fetch all shared debts for a group the user belongs to.
#[cfg(feature = "ssr")]
pub async fn list_group_debts(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Vec<SharedDebtWithDetails>, ServerFnError> {
    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member.count == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Not a member of this group",
        ));
    }

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
        user_id,
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    debts
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
        .collect()
}

/// Create a shared debt split between the selected group members.
#[cfg(feature = "ssr")]
pub async fn create_debt(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
    name: String,
    amount: String,
    member_ids: Vec<i64>,
) -> Result<i64, ServerFnError> {
    let name = validate_name(&name, 1, 255, "Debt name")?;

    let amount_decimal = validate_amount(&amount)?;

    if member_ids.is_empty() {
        return Err(ServerFnError::new(
            "At least one member must be selected to split the debt",
        ));
    }

    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member.count == 0 {
        return Err(ServerFnError::new(
            "You don't have permission to access this group",
        ));
    }

    validate_members_in_group(pool, group_id, &member_ids).await?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let amount_str = amount_decimal.to_string();
    let result = sqlx::query!(
        "INSERT INTO shared_debts (group_id, created_by, name, amount) VALUES (?, ?, ?, ?)",
        group_id,
        user_id,
        name,
        amount_str
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let debt_id = result.last_insert_rowid();

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

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(debt_id)
}

/// Update a shared debt (creator only) and replace its split members.
#[cfg(feature = "ssr")]
pub async fn update_debt(
    pool: &SqlitePool,
    user_id: i64,
    debt_id: i64,
    name: String,
    amount: String,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    let name = validate_name(&name, 1, 255, "Debt name")?;

    let amount_decimal = validate_amount(&amount)?;

    if member_ids.is_empty() {
        return Err(ServerFnError::new(
            "At least one member must be selected to split the debt",
        ));
    }

    let debt = sqlx::query!(
        "SELECT created_by, group_id FROM shared_debts WHERE id = ?",
        debt_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Shared debt not found"))?;

    if debt.created_by != user_id {
        return Err(ServerFnError::new("Only the creator can update this debt"));
    }

    validate_members_in_group(pool, debt.group_id, &member_ids).await?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

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

    sqlx::query!(
        "DELETE FROM shared_debt_user WHERE shared_debt_id = ?",
        debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

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

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Delete a shared debt (creator only).
#[cfg(feature = "ssr")]
pub async fn delete_debt(
    pool: &SqlitePool,
    user_id: i64,
    debt_id: i64,
) -> Result<(), ServerFnError> {
    let debt = sqlx::query!("SELECT created_by FROM shared_debts WHERE id = ?", debt_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Shared debt not found"))?;

    if debt.created_by != user_id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the creator can delete this debt",
        ));
    }

    sqlx::query!("DELETE FROM shared_debts WHERE id = ?", debt_id)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// List the user IDs participating in a shared debt.
#[cfg(feature = "ssr")]
pub async fn list_debt_members(
    pool: &SqlitePool,
    user_id: i64,
    debt_id: i64,
) -> Result<Vec<i64>, ServerFnError> {
    let _debt = sqlx::query!(
        r#"
        SELECT sd.id as "id!", sd.group_id as "group_id!", sd.amount
        FROM shared_debts sd
        INNER JOIN group_members gm ON sd.group_id = gm.group_id
        WHERE sd.id = ? AND gm.user_id = ?
        "#,
        debt_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Shared debt not found or access denied"))?;

    let members = sqlx::query!(
        "SELECT user_id FROM shared_debt_user WHERE shared_debt_id = ?",
        debt_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(members.into_iter().map(|row| row.user_id).collect())
}

/// List the per-user shares of a shared debt.
#[cfg(feature = "ssr")]
pub async fn list_debt_shares(
    pool: &SqlitePool,
    user_id: i64,
    debt_id: i64,
) -> Result<Vec<UserShare>, ServerFnError> {
    let debt = sqlx::query!(
        r#"
        SELECT sd.id as "id!", sd.group_id as "group_id!", sd.amount
        FROM shared_debts sd
        INNER JOIN group_members gm ON sd.group_id = gm.group_id
        WHERE sd.id = ? AND gm.user_id = ?
        "#,
        debt_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Shared debt not found or access denied"))?;

    let amount = debt
        .amount
        .parse::<Decimal>()
        .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

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
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_data: Vec<(i64, String)> = users
        .into_iter()
        .map(|row| (row.id, row.username))
        .collect();

    Ok(calculate_shares(amount, &user_data))
}

#[cfg(feature = "ssr")]
async fn validate_members_in_group(
    pool: &SqlitePool,
    group_id: i64,
    member_ids: &[i64],
) -> Result<(), ServerFnError> {
    for member_id in member_ids {
        let is_group_member = sqlx::query!(
            "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
            group_id,
            member_id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        if is_group_member.count == 0 {
            return Err(ServerFnError::new(
                "Some selected members are not part of this group",
            ));
        }
    }

    Ok(())
}
