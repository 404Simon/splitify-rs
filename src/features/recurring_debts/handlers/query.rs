//! Query operations for recurring debts (read-only operations)

use leptos::prelude::*;

use crate::features::recurring_debts::models::RecurringDebtWithDetails;

#[cfg(feature = "ssr")]
use crate::features::recurring_debts::models::Frequency;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use time::Date;

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
