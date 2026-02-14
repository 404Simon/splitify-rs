//! Update operations for recurring debts

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use crate::features::recurring_debts::models::Frequency;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use time::Date;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

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
