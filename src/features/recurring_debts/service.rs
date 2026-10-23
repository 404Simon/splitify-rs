//! Recurring debt business logic shared by the web server functions and the REST API.

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use rust_decimal::Decimal;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;
#[cfg(feature = "ssr")]
use time::Date;

#[cfg(feature = "ssr")]
use super::models::{Frequency, GeneratedInstance, RecurringDebtMember, RecurringDebtWithDetails};
#[cfg(feature = "ssr")]
use super::utils::calculate_next_occurrence;
#[cfg(feature = "ssr")]
use crate::features::shared_debts::utils::calculate_shares;

#[cfg(feature = "ssr")]
fn status_for(is_active: bool, end_date: Option<Date>, today: Date) -> String {
    if !is_active {
        "Paused".to_string()
    } else if let Some(end_date) = end_date {
        if today > end_date {
            "Expired".to_string()
        } else {
            "Active".to_string()
        }
    } else {
        "Active".to_string()
    }
}

/// Fetch all recurring debts for a group the user belongs to.
#[cfg(feature = "ssr")]
pub async fn list_for_group(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Vec<RecurringDebtWithDetails>, ServerFnError> {
    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .count
        > 0;

    if !is_member {
        return Err(ServerFnError::new("Not authorized"));
    }

    let today = time::OffsetDateTime::now_utc().date();

    let debts = sqlx::query!(
        r#"
        SELECT 
            rd.id as "id!",
            rd.group_id as "group_id!",
            rd.created_by as "created_by!",
            rd.name,
            rd.amount,
            rd.frequency,
            rd.start_date as "start_date!: String",
            rd.end_date as "end_date: String",
            rd.next_generation_date as "next_generation_date!: String",
            rd.is_active as "is_active!: bool",
            rd.created_at,
            rd.updated_at,
            u.username as creator_username,
            CASE WHEN rd.created_by = ? THEN 1 ELSE 0 END as "is_creator!: bool"
        FROM recurring_debts rd
        INNER JOIN users u ON rd.created_by = u.id
        WHERE rd.group_id = ?
        ORDER BY rd.created_at DESC
        "#,
        user_id,
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    debts
        .into_iter()
        .map(|debt| {
            let amount = debt
                .amount
                .parse::<Decimal>()
                .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

            let frequency = debt
                .frequency
                .parse::<Frequency>()
                .map_err(ServerFnError::new)?;

            let start_date = Date::parse(
                &debt.start_date,
                &time::format_description::well_known::Iso8601::DEFAULT,
            )
            .map_err(|e| ServerFnError::new(format!("Invalid start date: {}", e)))?;

            let end_date = if let Some(ed) = debt.end_date {
                Some(
                    Date::parse(&ed, &time::format_description::well_known::Iso8601::DEFAULT)
                        .map_err(|e| ServerFnError::new(format!("Invalid end date: {}", e)))?,
                )
            } else {
                None
            };

            let next_generation_date = Date::parse(
                &debt.next_generation_date,
                &time::format_description::well_known::Iso8601::DEFAULT,
            )
            .map_err(|e| ServerFnError::new(format!("Invalid next generation date: {}", e)))?;

            let status = status_for(debt.is_active, end_date, today);

            Ok(RecurringDebtWithDetails {
                id: debt.id,
                group_id: debt.group_id,
                created_by: debt.created_by,
                creator_username: debt.creator_username,
                name: debt.name,
                amount,
                frequency,
                start_date,
                end_date,
                next_generation_date,
                is_active: debt.is_active,
                created_at: debt.created_at,
                updated_at: debt.updated_at,
                is_creator: debt.is_creator,
                status,
            })
        })
        .collect()
}

/// Fetch a single recurring debt, verifying the user can access its group.
#[cfg(feature = "ssr")]
pub async fn get(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<RecurringDebtWithDetails, ServerFnError> {
    let today = time::OffsetDateTime::now_utc().date();

    let debt = sqlx::query!(
        r#"
        SELECT 
            rd.id as "id!",
            rd.group_id as "group_id!",
            rd.created_by as "created_by!",
            rd.name,
            rd.amount,
            rd.frequency,
            rd.start_date as "start_date!: String",
            rd.end_date as "end_date: String",
            rd.next_generation_date as "next_generation_date!: String",
            rd.is_active as "is_active!: bool",
            rd.created_at,
            rd.updated_at,
            u.username as creator_username,
            CASE WHEN rd.created_by = ? THEN 1 ELSE 0 END as "is_creator!: bool"
        FROM recurring_debts rd
        INNER JOIN users u ON rd.created_by = u.id
        INNER JOIN group_members gm ON rd.group_id = gm.group_id
        WHERE rd.id = ? AND gm.user_id = ?
        "#,
        user_id,
        recurring_debt_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Recurring debt not found or access denied"))?;

    let amount = debt
        .amount
        .parse::<Decimal>()
        .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

    let frequency = debt
        .frequency
        .parse::<Frequency>()
        .map_err(ServerFnError::new)?;

    let start_date = Date::parse(
        &debt.start_date,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .map_err(|e| ServerFnError::new(format!("Invalid start date: {}", e)))?;

    let end_date = if let Some(ed) = debt.end_date {
        Some(
            Date::parse(&ed, &time::format_description::well_known::Iso8601::DEFAULT)
                .map_err(|e| ServerFnError::new(format!("Invalid end date: {}", e)))?,
        )
    } else {
        None
    };

    let next_generation_date = Date::parse(
        &debt.next_generation_date,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .map_err(|e| ServerFnError::new(format!("Invalid next generation date: {}", e)))?;

    let status = status_for(debt.is_active, end_date, today);

    Ok(RecurringDebtWithDetails {
        id: debt.id,
        group_id: debt.group_id,
        created_by: debt.created_by,
        creator_username: debt.creator_username,
        name: debt.name,
        amount,
        frequency,
        start_date,
        end_date,
        next_generation_date,
        is_active: debt.is_active,
        created_at: debt.created_at,
        updated_at: debt.updated_at,
        is_creator: debt.is_creator,
        status,
    })
}

/// Create a recurring debt schedule for a group.
#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
    name: String,
    amount: String,
    frequency: String,
    start_date: String,
    end_date: Option<String>,
    member_ids: Vec<i64>,
) -> Result<i64, ServerFnError> {
    if name.trim().is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    let amount_decimal = amount
        .parse::<Decimal>()
        .map_err(|_| ServerFnError::new("Invalid amount"))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than 0"));
    }

    let _frequency_enum = frequency.parse::<Frequency>().map_err(ServerFnError::new)?;

    let start_date_parsed = Date::parse(
        &start_date,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .map_err(|_| ServerFnError::new("Invalid start date format (expected YYYY-MM-DD)"))?;

    let today = time::OffsetDateTime::now_utc().date();
    if start_date_parsed < today {
        return Err(ServerFnError::new(
            "Start date must be today or in the future",
        ));
    }

    let end_date_for_insert = end_date.clone();
    if let Some(ed) = end_date {
        let parsed = Date::parse(&ed, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| ServerFnError::new("Invalid end date format (expected YYYY-MM-DD)"))?;
        if parsed <= start_date_parsed {
            return Err(ServerFnError::new("End date must be after start date"));
        }
    }

    if member_ids.is_empty() {
        return Err(ServerFnError::new("At least one member must be selected"));
    }

    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .count
        > 0;

    if !is_member {
        return Err(ServerFnError::new("Not authorized"));
    }

    validate_members_in_group(pool, group_id, &member_ids).await?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let recurring_debt_id = sqlx::query!(
        r#"
        INSERT INTO recurring_debts (
            group_id, created_by, name, amount, frequency, 
            start_date, end_date, next_generation_date, is_active
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)
        "#,
        group_id,
        user_id,
        name,
        amount,
        frequency,
        start_date,
        end_date_for_insert,
        start_date
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .last_insert_rowid();

    for member_id in member_ids {
        sqlx::query!(
            "INSERT INTO recurring_debt_user (recurring_debt_id, user_id) VALUES (?, ?)",
            recurring_debt_id,
            member_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(recurring_debt_id)
}

/// Update a recurring debt (creator only) and replace its members.
#[cfg(feature = "ssr")]
#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
    name: String,
    amount: String,
    frequency: String,
    end_date: Option<String>,
    is_active: bool,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    if name.trim().is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    let amount_decimal = amount
        .parse::<Decimal>()
        .map_err(|_| ServerFnError::new("Invalid amount"))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than 0"));
    }

    let _frequency_enum = frequency.parse::<Frequency>().map_err(ServerFnError::new)?;

    let end_date_parsed = if let Some(ed) = &end_date {
        let parsed = Date::parse(ed, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| ServerFnError::new("Invalid end date format (expected YYYY-MM-DD)"))?;
        Some(parsed)
    } else {
        None
    };

    if member_ids.is_empty() {
        return Err(ServerFnError::new("At least one member must be selected"));
    }

    let debt = sqlx::query!(
        r#"
        SELECT rd.id, rd.group_id, rd.created_by, rd.start_date as "start_date!: String"
        FROM recurring_debts rd
        WHERE rd.id = ?
        "#,
        recurring_debt_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Recurring debt not found"))?;

    if debt.created_by != user_id {
        return Err(ServerFnError::new(
            "Only the creator can update this recurring debt",
        ));
    }

    if let Some(end_date_parsed) = end_date_parsed {
        let start_date = Date::parse(
            &debt.start_date,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .map_err(|e| ServerFnError::new(format!("Invalid start date in database: {}", e)))?;
        if end_date_parsed <= start_date {
            return Err(ServerFnError::new("End date must be after start date"));
        }
    }

    validate_members_in_group(pool, debt.group_id, &member_ids).await?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query!(
        r#"
        UPDATE recurring_debts
        SET name = ?, amount = ?, frequency = ?, end_date = ?, 
            is_active = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
        name,
        amount,
        frequency,
        end_date,
        is_active,
        recurring_debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query!(
        "DELETE FROM recurring_debt_user WHERE recurring_debt_id = ?",
        recurring_debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    for member_id in member_ids {
        sqlx::query!(
            "INSERT INTO recurring_debt_user (recurring_debt_id, user_id) VALUES (?, ?)",
            recurring_debt_id,
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

/// Delete a recurring debt (creator only).
#[cfg(feature = "ssr")]
pub async fn delete(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<(), ServerFnError> {
    let debt = sqlx::query!(
        r#"
        SELECT rd.id, rd.created_by
        FROM recurring_debts rd
        WHERE rd.id = ?
        "#,
        recurring_debt_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Recurring debt not found"))?;

    if debt.created_by != user_id {
        return Err(ServerFnError::new(
            "Only the creator can delete this recurring debt",
        ));
    }

    sqlx::query!(
        "DELETE FROM recurring_debts WHERE id = ?",
        recurring_debt_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Toggle a recurring debt's active state (creator only).
#[cfg(feature = "ssr")]
pub async fn toggle_active(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<bool, ServerFnError> {
    let debt = sqlx::query!(
        r#"
        SELECT rd.id, rd.created_by, rd.is_active as "is_active!: bool"
        FROM recurring_debts rd
        WHERE rd.id = ?
        "#,
        recurring_debt_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Recurring debt not found"))?;

    if debt.created_by != user_id {
        return Err(ServerFnError::new(
            "Only the creator can toggle this recurring debt",
        ));
    }

    let new_status = !debt.is_active;

    sqlx::query!(
        "UPDATE recurring_debts SET is_active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        new_status,
        recurring_debt_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(new_status)
}

/// Manually generate a shared debt from a recurring debt (creator only).
#[cfg(feature = "ssr")]
pub async fn generate_now(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<i64, ServerFnError> {
    let debt = sqlx::query!(
        r#"
        SELECT 
            rd.id, 
            rd.group_id, 
            rd.created_by, 
            rd.name,
            rd.amount,
            rd.frequency,
            rd.next_generation_date as "next_generation_date!: String",
            rd.is_active as "is_active!: bool"
        FROM recurring_debts rd
        WHERE rd.id = ?
        "#,
        recurring_debt_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Recurring debt not found"))?;

    if debt.created_by != user_id {
        return Err(ServerFnError::new(
            "Only the creator can manually generate debts",
        ));
    }

    if !debt.is_active {
        return Err(ServerFnError::new(
            "Cannot generate from an inactive recurring debt",
        ));
    }

    let frequency = debt
        .frequency
        .parse::<Frequency>()
        .map_err(ServerFnError::new)?;

    let next_generation_date = Date::parse(
        &debt.next_generation_date,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .map_err(|e| ServerFnError::new(format!("Invalid next generation date: {}", e)))?;

    let new_next_date = calculate_next_occurrence(next_generation_date, &frequency);

    let members = sqlx::query!(
        r#"
        SELECT user_id as "user_id!"
        FROM recurring_debt_user
        WHERE recurring_debt_id = ?
        "#,
        recurring_debt_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let member_ids: Vec<i64> = members.into_iter().map(|m| m.user_id).collect();

    if member_ids.is_empty() {
        return Err(ServerFnError::new(
            "No members found for this recurring debt",
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let shared_debt_id = sqlx::query!(
        r#"
        INSERT INTO shared_debts (group_id, created_by, name, amount, recurring_debt_id)
        VALUES (?, ?, ?, ?, ?)
        "#,
        debt.group_id,
        debt.created_by,
        debt.name,
        debt.amount,
        recurring_debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .last_insert_rowid();

    for member_id in member_ids {
        sqlx::query!(
            "INSERT INTO shared_debt_user (shared_debt_id, user_id) VALUES (?, ?)",
            shared_debt_id,
            member_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    let new_next_date_str = new_next_date.to_string();
    sqlx::query!(
        "UPDATE recurring_debts SET next_generation_date = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        new_next_date_str,
        recurring_debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_debt_id)
}

/// List the shared debts generated from a recurring debt.
#[cfg(feature = "ssr")]
pub async fn list_instances(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<Vec<GeneratedInstance>, ServerFnError> {
    ensure_access(pool, user_id, recurring_debt_id).await?;

    let instances = sqlx::query!(
        r#"
        SELECT 
            sd.id as "shared_debt_id!",
            sd.name as debt_name,
            sd.amount,
            sd.created_at
        FROM shared_debts sd
        WHERE sd.recurring_debt_id = ?
        ORDER BY sd.created_at DESC
        "#,
        recurring_debt_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    instances
        .into_iter()
        .map(|instance| {
            let amount = instance
                .amount
                .parse::<Decimal>()
                .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

            Ok(GeneratedInstance {
                shared_debt_id: instance.shared_debt_id,
                debt_name: instance.debt_name,
                amount,
                created_at: instance.created_at,
            })
        })
        .collect()
}

/// List the user IDs participating in a recurring debt.
#[cfg(feature = "ssr")]
pub async fn list_members(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<Vec<i64>, ServerFnError> {
    ensure_access(pool, user_id, recurring_debt_id).await?;

    let members = sqlx::query!(
        r#"
        SELECT user_id as "user_id!"
        FROM recurring_debt_user
        WHERE recurring_debt_id = ?
        "#,
        recurring_debt_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(members.into_iter().map(|m| m.user_id).collect())
}

/// List the per-user shares of a recurring debt.
#[cfg(feature = "ssr")]
pub async fn list_shares(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<Vec<RecurringDebtMember>, ServerFnError> {
    let debt = sqlx::query!(
        r#"
        SELECT rd.amount
        FROM recurring_debts rd
        INNER JOIN group_members gm ON rd.group_id = gm.group_id
        WHERE rd.id = ? AND gm.user_id = ?
        "#,
        recurring_debt_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Not authorized"))?;

    let amount = debt
        .amount
        .parse::<Decimal>()
        .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

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
    .fetch_all(pool)
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

#[cfg(feature = "ssr")]
async fn ensure_access(
    pool: &SqlitePool,
    user_id: i64,
    recurring_debt_id: i64,
) -> Result<(), ServerFnError> {
    let has_access = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM recurring_debts rd
        INNER JOIN group_members gm ON rd.group_id = gm.group_id
        WHERE rd.id = ? AND gm.user_id = ?
        "#,
        recurring_debt_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .count
        > 0;

    if !has_access {
        return Err(ServerFnError::new("Not authorized"));
    }

    Ok(())
}

#[cfg(feature = "ssr")]
async fn validate_members_in_group(
    pool: &SqlitePool,
    group_id: i64,
    member_ids: &[i64],
) -> Result<(), ServerFnError> {
    for member_id in member_ids {
        let is_member = sqlx::query!(
            "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
            group_id,
            member_id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .count
            > 0;

        if !is_member {
            return Err(ServerFnError::new(format!(
                "User {} is not a member of this group",
                member_id
            )));
        }
    }

    Ok(())
}
