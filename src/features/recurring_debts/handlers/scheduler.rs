#[cfg(feature = "ssr")]
use rust_decimal::Decimal;
#[cfg(feature = "ssr")]
use time::Date;

#[cfg(feature = "ssr")]
use crate::features::recurring_debts::models::{Frequency, RecurringDebt};
#[cfg(feature = "ssr")]
use crate::features::recurring_debts::utils::{calculate_next_occurrence, should_generate};

#[cfg(feature = "ssr")]
pub async fn process_due_recurring_debts_internal(
    pool: sqlx::SqlitePool,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let today = time::OffsetDateTime::now_utc().date();
    let today_str = today.to_string();

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
    .await?;

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

        let amount = match debt_row.amount.parse::<Decimal>() {
            Ok(a) => a,
            Err(e) => {
                eprintln!(
                    "Error parsing amount for recurring debt {}: {}",
                    debt_row.id, e
                );
                continue;
            }
        };

        let recurring_debt = RecurringDebt {
            id: debt_row.id,
            group_id: debt_row.group_id,
            created_by: debt_row.created_by,
            name: debt_row.name.clone(),
            amount,
            frequency: frequency.clone(),
            start_date,
            end_date,
            next_generation_date,
            is_active: debt_row.is_active,
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        };

        if !should_generate(&recurring_debt, today) {
            continue;
        }

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

        let new_next_date = calculate_next_occurrence(next_generation_date, &frequency);

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
            }
        }

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
            continue;
        }

        if let Err(e) = tx.commit().await {
            eprintln!(
                "Error committing transaction for recurring debt {}: {}",
                debt_row.id, e
            );
            continue;
        }

        generated_count += 1;
        tracing::info!(
            shared_debt_id = shared_debt_id,
            recurring_debt_id = debt_row.id,
            "Generated shared debt from recurring debt"
        );
    }

    Ok(generated_count)
}
