//! Create operations for recurring debts

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use rust_decimal::Decimal;
#[cfg(feature = "ssr")]
use time::Date;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
#[cfg(feature = "ssr")]
use crate::features::recurring_debts::models::Frequency;

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
