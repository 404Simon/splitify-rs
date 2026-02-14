//! Balance calculations combining shared debts and transactions

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use rust_decimal::Decimal;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
use crate::features::transactions::models::UserBalance;
#[cfg(feature = "ssr")]
use crate::features::transactions::models::{DebtRelationship, NetType, RelationshipType};

/// Calculate user debts for a group (combines shared debts and transactions)
#[server(CalculateUserDebts)]
pub async fn calculate_user_debts(group_id: i64) -> Result<Vec<UserBalance>, ServerFnError> {
    use std::collections::HashMap;

    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Check user is member of group
    let is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user.id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member == 0 {
        return Err(ServerFnError::new(
            "You are not a member of this group".to_string(),
        ));
    }

    // Get all group members
    let members = sqlx::query!(
        r#"
        SELECT u.id as "id!", u.username
        FROM users u
        JOIN group_members gm ON u.id = gm.user_id
        WHERE gm.group_id = ?
        "#,
        group_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Initialize debt matrix (who owes whom)
    let mut debts: HashMap<i64, HashMap<i64, Decimal>> = HashMap::new();
    for member in &members {
        debts.insert(member.id, HashMap::new());
    }

    // Calculate debts from shared debts
    calculate_shared_debt_contributions(&pool, group_id, &mut debts).await?;

    // Factor in direct transactions
    calculate_transaction_contributions(&pool, group_id, &mut debts).await?;

    // Build UserBalance objects
    let mut balances = Vec::new();
    for member in &members {
        let user_debts = match debts.get(&member.id) {
            Some(debts) => debts,
            None => continue, // Skip if somehow debt map is incomplete
        };

        let mut relationships = Vec::new();
        let mut total_owed = Decimal::ZERO;
        let mut total_owing = Decimal::ZERO;

        for (other_user_id, amount) in user_debts {
            if *amount == Decimal::ZERO {
                continue;
            }

            let other_username = members
                .iter()
                .find(|m| m.id == *other_user_id)
                .map(|m| m.username.clone())
                .unwrap_or_default();

            if *amount > Decimal::ZERO {
                // User owes money to other_user
                relationships.push(DebtRelationship {
                    other_user_id: *other_user_id,
                    other_username,
                    amount: amount.round_dp(2).to_string(),
                    relationship_type: RelationshipType::Owes,
                });
                total_owing += *amount;
            } else {
                // Other user owes money to user (amount is negative)
                relationships.push(DebtRelationship {
                    other_user_id: *other_user_id,
                    other_username,
                    amount: amount.abs().round_dp(2).to_string(),
                    relationship_type: RelationshipType::Owed,
                });
                total_owed += amount.abs();
            }
        }

        let net_amount = total_owed - total_owing;
        let net_type = if net_amount > Decimal::ZERO {
            NetType::Positive
        } else if net_amount < Decimal::ZERO {
            NetType::Negative
        } else {
            NetType::Neutral
        };

        balances.push(UserBalance {
            user_id: member.id,
            username: member.username.clone(),
            relationships,
            total_owed: total_owed.round_dp(2).to_string(),
            total_owing: total_owing.round_dp(2).to_string(),
            net_amount: net_amount.abs().round_dp(2).to_string(),
            net_type,
        });
    }

    Ok(balances)
}

#[cfg(feature = "ssr")]
async fn calculate_shared_debt_contributions(
    pool: &sqlx::SqlitePool,
    group_id: i64,
    debts: &mut std::collections::HashMap<i64, std::collections::HashMap<i64, Decimal>>,
) -> Result<(), ServerFnError> {
    use std::str::FromStr;
    // Fetch all shared debts for the group
    let shared_debts = sqlx::query!(
        r#"
        SELECT id as "id!", created_by as "created_by!", amount
        FROM shared_debts
        WHERE group_id = ?
        "#,
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    for debt in shared_debts {
        let creator_id = debt.created_by;
        let total_amount =
            Decimal::from_str(&debt.amount).map_err(|e| ServerFnError::new(e.to_string()))?;

        // Get participants in this shared debt
        let participants = sqlx::query!(
            "SELECT user_id FROM shared_debt_user WHERE shared_debt_id = ?",
            debt.id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        if participants.is_empty() {
            continue;
        }

        let share_per_user = total_amount / Decimal::from(participants.len());

        // Each participant (except creator) owes their share to the creator
        for participant in participants {
            let user_id = participant.user_id;
            if user_id == creator_id {
                continue; // Creator doesn't owe themselves
            }

            // User owes creator
            if let Some(user_debts) = debts.get_mut(&user_id) {
                *user_debts.entry(creator_id).or_insert(Decimal::ZERO) += share_per_user;
            }

            // Creator is owed by user (negative debt)
            if let Some(creator_debts) = debts.get_mut(&creator_id) {
                *creator_debts.entry(user_id).or_insert(Decimal::ZERO) -= share_per_user;
            }
        }
    }

    Ok(())
}

#[cfg(feature = "ssr")]
async fn calculate_transaction_contributions(
    pool: &sqlx::SqlitePool,
    group_id: i64,
    debts: &mut std::collections::HashMap<i64, std::collections::HashMap<i64, Decimal>>,
) -> Result<(), ServerFnError> {
    use std::str::FromStr;
    // Fetch all transactions for the group
    let transactions = sqlx::query!(
        r#"
        SELECT payer_id as "payer_id!", recipient_id as "recipient_id!", amount
        FROM transactions
        WHERE group_id = ?
        "#,
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    for transaction in transactions {
        let payer_id = transaction.payer_id;
        let recipient_id = transaction.recipient_id;
        let amount = Decimal::from_str(&transaction.amount)
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // When payer pays recipient, payer's debt to recipient decreases
        // Payer owes less (or is owed more)
        if let Some(payer_debts) = debts.get_mut(&payer_id) {
            *payer_debts.entry(recipient_id).or_insert(Decimal::ZERO) -= amount;
        }

        // Recipient is owed less by payer (or owes more to payer)
        if let Some(recipient_debts) = debts.get_mut(&recipient_id) {
            *recipient_debts.entry(payer_id).or_insert(Decimal::ZERO) += amount;
        }
    }

    Ok(())
}
