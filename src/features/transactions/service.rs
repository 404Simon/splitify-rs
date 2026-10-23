//! Transaction and balance business logic shared by the web server functions and
//! the REST API.

#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use rust_decimal::Decimal;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

#[cfg(feature = "ssr")]
use super::models::{
    DebtRelationship, NetType, RelationshipType, TransactionWithDetails, UserBalance,
};

/// Fetch a single transaction after verifying the user belongs to its group.
#[cfg(feature = "ssr")]
pub async fn get_transaction(
    pool: &SqlitePool,
    user_id: i64,
    transaction_id: i64,
) -> Result<TransactionWithDetails, ServerFnError> {
    let record = sqlx::query!(
        r#"
        SELECT 
            t.id as "id!",
            t.group_id as "group_id!",
            t.payer_id as "payer_id!",
            payer.username as payer_username,
            t.recipient_id as "recipient_id!",
            recipient.username as recipient_username,
            t.amount,
            t.description,
            t.created_at,
            t.updated_at
        FROM transactions t
        JOIN users payer ON t.payer_id = payer.id
        JOIN users recipient ON t.recipient_id = recipient.id
        WHERE t.id = ?
        "#,
        transaction_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found"))?;

    verify_group_membership(pool, user_id, record.group_id).await?;

    Ok(TransactionWithDetails {
        id: record.id,
        group_id: record.group_id,
        payer_id: record.payer_id,
        payer_username: record.payer_username,
        recipient_id: record.recipient_id,
        recipient_username: record.recipient_username,
        amount: record.amount,
        description: record.description,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

/// Fetch all transactions for a group the user belongs to.
#[cfg(feature = "ssr")]
pub async fn list_group_transactions(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Vec<TransactionWithDetails>, ServerFnError> {
    verify_group_membership(pool, user_id, group_id).await?;

    let records = sqlx::query!(
        r#"
        SELECT 
            t.id as "id!",
            t.group_id as "group_id!",
            t.payer_id as "payer_id!",
            payer.username as payer_username,
            t.recipient_id as "recipient_id!",
            recipient.username as recipient_username,
            t.amount,
            t.description,
            t.created_at,
            t.updated_at
        FROM transactions t
        JOIN users payer ON t.payer_id = payer.id
        JOIN users recipient ON t.recipient_id = recipient.id
        WHERE t.group_id = ?
        ORDER BY t.created_at DESC
        "#,
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(records
        .into_iter()
        .map(|r| TransactionWithDetails {
            id: r.id,
            group_id: r.group_id,
            payer_id: r.payer_id,
            payer_username: r.payer_username,
            recipient_id: r.recipient_id,
            recipient_username: r.recipient_username,
            amount: r.amount,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// Create a transaction where the current user is the payer.
#[cfg(feature = "ssr")]
pub async fn create_transaction(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
    recipient_id: i64,
    amount: String,
    description: Option<String>,
) -> Result<i64, ServerFnError> {
    let amount_decimal = Decimal::from_str(&amount)
        .map_err(|_| ServerFnError::new("Invalid amount format".to_string()))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new(
            "Amount must be greater than 0".to_string(),
        ));
    }

    verify_group_membership(pool, user_id, group_id).await?;

    let recipient_is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        recipient_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if recipient_is_member == 0 {
        return Err(ServerFnError::new(
            "Recipient is not a member of this group".to_string(),
        ));
    }

    if user_id == recipient_id {
        return Err(ServerFnError::new(
            "Cannot create transaction with yourself".to_string(),
        ));
    }

    let amount_str = amount_decimal.round_dp(2).to_string();

    let result = sqlx::query!(
        r#"
        INSERT INTO transactions (group_id, payer_id, recipient_id, amount, description)
        VALUES (?, ?, ?, ?, ?)
        "#,
        group_id,
        user_id,
        recipient_id,
        amount_str,
        description
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
}

/// Update a transaction (payer only), deriving its group from the stored row.
#[cfg(feature = "ssr")]
pub async fn update_transaction(
    pool: &SqlitePool,
    user_id: i64,
    transaction_id: i64,
    recipient_id: i64,
    amount: String,
    description: Option<String>,
) -> Result<(), ServerFnError> {
    let amount_decimal =
        Decimal::from_str(&amount).map_err(|_| ServerFnError::new("Invalid amount format"))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than 0"));
    }

    let existing = sqlx::query!(
        "SELECT payer_id, group_id FROM transactions WHERE id = ?",
        transaction_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found"))?;

    if existing.payer_id != user_id {
        return Err(ServerFnError::new(
            "You can only edit your own transactions",
        ));
    }

    let recipient_is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        existing.group_id,
        recipient_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if recipient_is_member == 0 {
        return Err(ServerFnError::new(
            "Recipient is not a member of this group",
        ));
    }

    if user_id == recipient_id {
        return Err(ServerFnError::new(
            "Cannot create transaction with yourself",
        ));
    }

    let amount_str = amount_decimal.round_dp(2).to_string();

    sqlx::query!(
        r#"
        UPDATE transactions
        SET recipient_id = ?, amount = ?, description = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
        recipient_id,
        amount_str,
        description,
        transaction_id
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Delete a transaction (payer only).
#[cfg(feature = "ssr")]
pub async fn delete_transaction(
    pool: &SqlitePool,
    user_id: i64,
    transaction_id: i64,
) -> Result<(), ServerFnError> {
    let existing = sqlx::query!(
        "SELECT payer_id FROM transactions WHERE id = ?",
        transaction_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found"))?;

    if existing.payer_id != user_id {
        return Err(ServerFnError::new(
            "You can only delete your own transactions",
        ));
    }

    sqlx::query!("DELETE FROM transactions WHERE id = ?", transaction_id)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Calculate per-user balances for a group from shared debts and transactions.
#[cfg(feature = "ssr")]
pub async fn group_balances(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<Vec<UserBalance>, ServerFnError> {
    verify_group_membership(pool, user_id, group_id).await?;

    let members = sqlx::query!(
        r#"
        SELECT u.id as "id!", u.username
        FROM users u
        JOIN group_members gm ON u.id = gm.user_id
        WHERE gm.group_id = ?
        "#,
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut debts: HashMap<i64, HashMap<i64, Decimal>> = HashMap::new();
    for member in &members {
        debts.insert(member.id, HashMap::new());
    }

    calculate_shared_debt_contributions(pool, group_id, &mut debts).await?;

    calculate_transaction_contributions(pool, group_id, &mut debts).await?;

    let mut balances = Vec::new();
    for member in &members {
        let user_debts = match debts.get(&member.id) {
            Some(debts) => debts,
            None => continue,
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
                relationships.push(DebtRelationship {
                    other_user_id: *other_user_id,
                    other_username,
                    amount: amount.round_dp(2).to_string(),
                    relationship_type: RelationshipType::Owes,
                });
                total_owing += *amount;
            } else {
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
async fn verify_group_membership(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<(), ServerFnError> {
    let is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member == 0 {
        return Err(ServerFnError::new(
            "You are not a member of this group".to_string(),
        ));
    }

    Ok(())
}

#[cfg(feature = "ssr")]
async fn calculate_shared_debt_contributions(
    pool: &SqlitePool,
    group_id: i64,
    debts: &mut HashMap<i64, HashMap<i64, Decimal>>,
) -> Result<(), ServerFnError> {
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

        for participant in participants {
            let user_id = participant.user_id;
            if user_id == creator_id {
                continue;
            }

            if let Some(user_debts) = debts.get_mut(&user_id) {
                *user_debts.entry(creator_id).or_insert(Decimal::ZERO) += share_per_user;
            }

            if let Some(creator_debts) = debts.get_mut(&creator_id) {
                *creator_debts.entry(user_id).or_insert(Decimal::ZERO) -= share_per_user;
            }
        }
    }

    Ok(())
}

#[cfg(feature = "ssr")]
async fn calculate_transaction_contributions(
    pool: &SqlitePool,
    group_id: i64,
    debts: &mut HashMap<i64, HashMap<i64, Decimal>>,
) -> Result<(), ServerFnError> {
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

        if let Some(payer_debts) = debts.get_mut(&payer_id) {
            *payer_debts.entry(recipient_id).or_insert(Decimal::ZERO) -= amount;
        }

        if let Some(recipient_debts) = debts.get_mut(&recipient_id) {
            *recipient_debts.entry(payer_id).or_insert(Decimal::ZERO) += amount;
        }
    }

    Ok(())
}
