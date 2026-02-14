use leptos::prelude::*;

use super::models::{GeneratedInstance, RecurringDebtMember, RecurringDebtWithDetails};

#[cfg(feature = "ssr")]
use super::models::Frequency;

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use time::Date;

#[cfg(feature = "ssr")]
use super::utils::{calculate_next_occurrence, should_generate};

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use crate::features::shared_debts::utils::calculate_shares;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Server function: Get all recurring debts for a group
#[server(GetRecurringDebts)]
pub async fn get_recurring_debts(
    group_id: i64,
) -> Result<Vec<RecurringDebtWithDetails>, ServerFnError> {
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
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .count
        > 0;

    if !is_member {
        return Err(ServerFnError::new("Not authorized"));
    }

    let today = time::OffsetDateTime::now_utc().date();

    // Get all recurring debts for the group
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
        user.id,
        group_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut result = Vec::new();
    for debt in debts {
        let amount = debt
            .amount
            .parse::<Decimal>()
            .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

        let frequency = debt
            .frequency
            .parse::<Frequency>()
            .map_err(|e| ServerFnError::new(e))?;

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

        // Determine status
        let status = if !debt.is_active {
            "Paused".to_string()
        } else if let Some(end_date) = end_date {
            if today > end_date {
                "Expired".to_string()
            } else {
                "Active".to_string()
            }
        } else {
            "Active".to_string()
        };

        result.push(RecurringDebtWithDetails {
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
        });
    }

    Ok(result)
}

/// Server function: Get a specific recurring debt
#[server(GetRecurringDebt)]
pub async fn get_recurring_debt(
    recurring_debt_id: i64,
) -> Result<RecurringDebtWithDetails, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    let today = time::OffsetDateTime::now_utc().date();

    // Get the recurring debt and verify user has access
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
        user.id,
        recurring_debt_id,
        user.id
    )
    .fetch_optional(&pool)
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
        .map_err(|e| ServerFnError::new(e))?;

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

    // Determine status
    let status = if !debt.is_active {
        "Paused".to_string()
    } else if let Some(end_date) = end_date {
        if today > end_date {
            "Expired".to_string()
        } else {
            "Active".to_string()
        }
    } else {
        "Active".to_string()
    };

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

/// Server function: Create a new recurring debt
#[server(CreateRecurringDebt)]
pub async fn create_recurring_debt(
    group_id: i64,
    name: String,
    amount: String,
    frequency: String,
    start_date: String,
    end_date: Option<String>,
    member_ids: Vec<i64>,
) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Validate input
    if name.trim().is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    let amount_decimal = amount
        .parse::<Decimal>()
        .map_err(|_| ServerFnError::new("Invalid amount"))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than 0"));
    }

    let _frequency_enum = frequency
        .parse::<Frequency>()
        .map_err(|e| ServerFnError::new(e))?;

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
    let _end_date_parsed = if let Some(ed) = end_date {
        let parsed = Date::parse(&ed, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| ServerFnError::new("Invalid end date format (expected YYYY-MM-DD)"))?;
        if parsed <= start_date_parsed {
            return Err(ServerFnError::new("End date must be after start date"));
        }
        Some(parsed)
    } else {
        None
    };

    if member_ids.is_empty() {
        return Err(ServerFnError::new("At least one member must be selected"));
    }

    // Check if user is a member of the group
    let is_member = sqlx::query!(
        "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .count
        > 0;

    if !is_member {
        return Err(ServerFnError::new("Not authorized"));
    }

    // Validate that all selected members are in the group
    for member_id in &member_ids {
        let is_member = sqlx::query!(
            "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
            group_id,
            member_id
        )
        .fetch_one(&pool)
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

    // Begin transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Insert recurring debt
    let recurring_debt_id = sqlx::query!(
        r#"
        INSERT INTO recurring_debts (
            group_id, created_by, name, amount, frequency, 
            start_date, end_date, next_generation_date, is_active
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)
        "#,
        group_id,
        user.id,
        name,
        amount,
        frequency,
        start_date,
        end_date_for_insert,
        start_date // next_generation_date = start_date initially
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .last_insert_rowid();

    // Insert members into pivot table
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

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(recurring_debt_id)
}

/// Server function: Update a recurring debt
#[server(UpdateRecurringDebt)]
pub async fn update_recurring_debt(
    recurring_debt_id: i64,
    name: String,
    amount: String,
    frequency: String,
    end_date: Option<String>,
    is_active: bool,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Validate input
    if name.trim().is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    let amount_decimal = amount
        .parse::<Decimal>()
        .map_err(|_| ServerFnError::new("Invalid amount"))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than 0"));
    }

    let _frequency_enum = frequency
        .parse::<Frequency>()
        .map_err(|e| ServerFnError::new(e))?;

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

    // Get the recurring debt and verify user is the creator
    let debt = sqlx::query!(
        r#"
        SELECT rd.id, rd.group_id, rd.created_by, rd.start_date as "start_date!: String"
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
            "Only the creator can update this recurring debt",
        ));
    }

    // Validate end_date against start_date
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

    // Validate that all selected members are in the group
    for member_id in &member_ids {
        let is_member = sqlx::query!(
            "SELECT COUNT(*) as \"count!\" FROM group_members WHERE group_id = ? AND user_id = ?",
            debt.group_id,
            member_id
        )
        .fetch_one(&pool)
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

    // Begin transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Update recurring debt
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

    // Delete existing members
    sqlx::query!(
        "DELETE FROM recurring_debt_user WHERE recurring_debt_id = ?",
        recurring_debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Insert new members
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

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

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

/// Server function: Get generated instances (SharedDebts) from a recurring debt
#[server(GetGeneratedInstances)]
pub async fn get_generated_instances(
    recurring_debt_id: i64,
) -> Result<Vec<GeneratedInstance>, ServerFnError> {
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

    // Get all generated shared debts
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
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut result = Vec::new();
    for instance in instances {
        let amount = instance
            .amount
            .parse::<Decimal>()
            .map_err(|e| ServerFnError::new(format!("Invalid amount: {}", e)))?;

        result.push(GeneratedInstance {
            shared_debt_id: instance.shared_debt_id,
            debt_name: instance.debt_name,
            amount,
            created_at: instance.created_at,
        });
    }

    Ok(result)
}

/// Server function: Manually generate a shared debt from a recurring debt
#[server(GenerateNow)]
pub async fn generate_now(recurring_debt_id: i64) -> Result<i64, ServerFnError> {
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
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Recurring debt not found"))?;

    if debt.created_by != user.id {
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
        .map_err(|e| ServerFnError::new(e))?;

    let next_generation_date = Date::parse(
        &debt.next_generation_date,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .map_err(|e| ServerFnError::new(format!("Invalid next generation date: {}", e)))?;

    // Calculate new next_generation_date
    let new_next_date = calculate_next_occurrence(next_generation_date, &frequency);

    // Get members of the recurring debt
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

    let member_ids: Vec<i64> = members.into_iter().map(|m| m.user_id).collect();

    if member_ids.is_empty() {
        return Err(ServerFnError::new(
            "No members found for this recurring debt",
        ));
    }

    // Begin transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Create SharedDebt
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

    // Insert members into shared_debt_user
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

    // Update next_generation_date
    let new_next_date_str = new_next_date.to_string();
    sqlx::query!(
        "UPDATE recurring_debts SET next_generation_date = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        new_next_date_str,
        recurring_debt_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(shared_debt_id)
}

/// Server function: Process all due recurring debts (background job)
#[server(ProcessDueRecurringDebts)]
pub async fn process_due_recurring_debts() -> Result<usize, ServerFnError> {
    use sqlx::SqlitePool;

    let pool = expect_context::<SqlitePool>();
    let today = time::OffsetDateTime::now_utc().date();
    let today_str = today.to_string();

    // Get all active recurring debts that are due for generation
    let debts = sqlx::query!(
        r#"
        SELECT 
            id as "id!",
            group_id as "group_id!",
            created_by as "created_by!",
            name,
            amount,
            frequency,
            start_date as "start_date!: String",
            end_date as "end_date: String",
            next_generation_date as "next_generation_date!: String",
            is_active as "is_active!: bool"
        FROM recurring_debts
        WHERE is_active = 1 
        AND next_generation_date <= ?
        AND (end_date IS NULL OR end_date >= ?)
        "#,
        today_str,
        today_str
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut generated_count = 0;

    for debt_row in debts {
        let frequency = match debt_row.frequency.parse::<Frequency>() {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Error parsing frequency for recurring debt {}: {}",
                    debt_row.id, e
                );
                continue;
            }
        };

        let next_generation_date = match Date::parse(
            &debt_row.next_generation_date,
            &time::format_description::well_known::Iso8601::DEFAULT,
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!(
                    "Error parsing next_generation_date for recurring debt {}: {}",
                    debt_row.id, e
                );
                continue;
            }
        };

        let end_date = if let Some(ed) = debt_row.end_date {
            match Date::parse(&ed, &time::format_description::well_known::Iso8601::DEFAULT) {
                Ok(d) => Some(d),
                Err(e) => {
                    eprintln!(
                        "Error parsing end_date for recurring debt {}: {}",
                        debt_row.id, e
                    );
                    continue;
                }
            }
        } else {
            None
        };

        let start_date = match Date::parse(
            &debt_row.start_date,
            &time::format_description::well_known::Iso8601::DEFAULT,
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!(
                    "Error parsing start_date for recurring debt {}: {}",
                    debt_row.id, e
                );
                continue;
            }
        };

        // Double-check eligibility using should_generate
        let recurring_debt = super::models::RecurringDebt {
            id: debt_row.id,
            group_id: debt_row.group_id,
            created_by: debt_row.created_by,
            name: debt_row.name.clone(),
            amount: match debt_row.amount.parse::<Decimal>() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!(
                        "Error parsing amount for recurring debt {}: {}",
                        debt_row.id, e
                    );
                    continue;
                }
            },
            frequency: frequency.clone(),
            start_date,
            end_date,
            next_generation_date,
            is_active: debt_row.is_active,
            created_at: time::OffsetDateTime::now_utc(), // Placeholder
            updated_at: time::OffsetDateTime::now_utc(), // Placeholder
        };

        if !should_generate(&recurring_debt, today) {
            continue;
        }

        // Get members
        let members = match sqlx::query!(
            r#"
            SELECT user_id as "user_id!"
            FROM recurring_debt_user
            WHERE recurring_debt_id = ?
            "#,
            debt_row.id
        )
        .fetch_all(&pool)
        .await
        {
            Ok(m) => m,
            Err(e) => {
                eprintln!(
                    "Error fetching members for recurring debt {}: {}",
                    debt_row.id, e
                );
                continue;
            }
        };

        let member_ids: Vec<i64> = members.into_iter().map(|m| m.user_id).collect();

        if member_ids.is_empty() {
            eprintln!("No members found for recurring debt {}", debt_row.id);
            continue;
        }

        // Calculate new next_generation_date
        let new_next_date = calculate_next_occurrence(next_generation_date, &frequency);

        // Begin transaction for this debt generation
        let mut tx = match pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "Error starting transaction for recurring debt {}: {}",
                    debt_row.id, e
                );
                continue;
            }
        };

        // Create SharedDebt
        let shared_debt_id = match sqlx::query!(
            r#"
            INSERT INTO shared_debts (group_id, created_by, name, amount, recurring_debt_id)
            VALUES (?, ?, ?, ?, ?)
            "#,
            debt_row.group_id,
            debt_row.created_by,
            debt_row.name,
            debt_row.amount,
            debt_row.id
        )
        .execute(&mut *tx)
        .await
        {
            Ok(result) => result.last_insert_rowid(),
            Err(e) => {
                eprintln!(
                    "Error creating shared debt for recurring debt {}: {}",
                    debt_row.id, e
                );
                continue;
            }
        };

        // Insert members into shared_debt_user
        for member_id in member_ids {
            if let Err(e) = sqlx::query!(
                "INSERT INTO shared_debt_user (shared_debt_id, user_id) VALUES (?, ?)",
                shared_debt_id,
                member_id
            )
            .execute(&mut *tx)
            .await
            {
                eprintln!(
                    "Error inserting member {} for shared debt {}: {}",
                    member_id, shared_debt_id, e
                );
                // Continue anyway - partial data is better than nothing
            }
        }

        // Update next_generation_date
        let new_next_date_str = new_next_date.to_string();
        if let Err(e) = sqlx::query!(
            "UPDATE recurring_debts SET next_generation_date = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            new_next_date_str,
            debt_row.id
        )
        .execute(&mut *tx)
        .await
        {
            eprintln!("Error updating next_generation_date for recurring debt {}: {}", debt_row.id, e);
            // Don't commit if we can't update the next generation date
            continue;
        }

        // Commit transaction
        if let Err(e) = tx.commit().await {
            eprintln!(
                "Error committing transaction for recurring debt {}: {}",
                debt_row.id, e
            );
            continue;
        }

        generated_count += 1;
        leptos::logging::log!(
            "Generated shared debt {} from recurring debt {}",
            shared_debt_id,
            debt_row.id
        );
    }

    Ok(generated_count)
}
