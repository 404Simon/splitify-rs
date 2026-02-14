//! Operations for managing generated instances (SharedDebts)

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
use crate::features::recurring_debts::models::GeneratedInstance;
#[cfg(feature = "ssr")]
use crate::features::recurring_debts::utils::calculate_next_occurrence;

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
        .map_err(ServerFnError::new)?;

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
