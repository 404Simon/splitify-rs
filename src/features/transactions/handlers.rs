use leptos::prelude::*;

use super::models::{Transaction, TransactionWithDetails, UserBalance};

#[cfg(feature = "ssr")]
use super::models::{DebtRelationship, NetType, RelationshipType};

#[cfg(feature = "ssr")]
use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
use leptos_axum::extract;

#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Get a single transaction by ID
#[server(GetTransaction)]
pub async fn get_transaction(
    group_id: i64,
    transaction_id: i64,
) -> Result<Transaction, ServerFnError> {
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

    // Fetch transaction
    let record = sqlx::query!(
        r#"
        SELECT 
            id as "id!",
            group_id as "group_id!",
            payer_id as "payer_id!",
            recipient_id as "recipient_id!",
            amount,
            description,
            created_at,
            updated_at
        FROM transactions
        WHERE id = ? AND group_id = ?
        "#,
        transaction_id,
        group_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found".to_string()))?;

    Ok(Transaction {
        id: record.id,
        group_id: record.group_id,
        payer_id: record.payer_id,
        recipient_id: record.recipient_id,
        amount: record.amount,
        description: record.description,
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

/// Get all transactions for a group
#[server(GetGroupTransactions)]
pub async fn get_group_transactions(
    group_id: i64,
) -> Result<Vec<TransactionWithDetails>, ServerFnError> {
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

    // Fetch transactions with payer and recipient usernames
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
    .fetch_all(&pool)
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

/// Create a new transaction
#[server(CreateTransaction)]
pub async fn create_transaction(
    group_id: i64,
    recipient_id: i64,
    amount: String,
    description: Option<String>,
) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;
    use std::str::FromStr;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Validate amount
    let amount_decimal = Decimal::from_str(&amount)
        .map_err(|_| ServerFnError::new("Invalid amount format".to_string()))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new(
            "Amount must be greater than 0".to_string(),
        ));
    }

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

    // Check recipient is member of group
    let recipient_is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        recipient_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if recipient_is_member == 0 {
        return Err(ServerFnError::new(
            "Recipient is not a member of this group".to_string(),
        ));
    }

    // Don't allow self-transactions
    if user.id == recipient_id {
        return Err(ServerFnError::new(
            "Cannot create transaction with yourself".to_string(),
        ));
    }

    // Store amount rounded to 2 decimal places
    let amount_str = amount_decimal.round_dp(2).to_string();

    // Insert transaction
    let result = sqlx::query!(
        r#"
        INSERT INTO transactions (group_id, payer_id, recipient_id, amount, description)
        VALUES (?, ?, ?, ?, ?)
        "#,
        group_id,
        user.id,
        recipient_id,
        amount_str,
        description
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
}

/// Update an existing transaction
#[server(UpdateTransaction)]
pub async fn update_transaction(
    group_id: i64,
    transaction_id: i64,
    recipient_id: i64,
    amount: String,
    description: Option<String>,
) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    use std::str::FromStr;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Validate amount
    let amount_decimal =
        Decimal::from_str(&amount).map_err(|_| ServerFnError::new("Invalid amount format"))?;

    if amount_decimal <= Decimal::ZERO {
        return Err(ServerFnError::new("Amount must be greater than 0"));
    }

    // Check transaction exists and user is the payer
    let existing = sqlx::query!(
        "SELECT payer_id FROM transactions WHERE id = ? AND group_id = ?",
        transaction_id,
        group_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found"))?;

    if existing.payer_id != user.id {
        return Err(ServerFnError::new(
            "You can only edit your own transactions",
        ));
    }

    // Check recipient is member of group
    let recipient_is_member = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM group_members WHERE group_id = ? AND user_id = ?",
        group_id,
        recipient_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if recipient_is_member == 0 {
        return Err(ServerFnError::new(
            "Recipient is not a member of this group",
        ));
    }

    // Don't allow self-transactions
    if user.id == recipient_id {
        return Err(ServerFnError::new(
            "Cannot create transaction with yourself",
        ));
    }

    // Store amount rounded to 2 decimal places
    let amount_str = amount_decimal.round_dp(2).to_string();

    // Update transaction
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
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Delete a transaction
#[server(DeleteTransaction)]
pub async fn delete_transaction(group_id: i64, transaction_id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Check transaction exists and user is the payer
    let existing = sqlx::query!(
        "SELECT payer_id FROM transactions WHERE id = ? AND group_id = ?",
        transaction_id,
        group_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("Transaction not found"))?;

    if existing.payer_id != user.id {
        return Err(ServerFnError::new(
            "You can only delete your own transactions",
        ));
    }

    // Delete transaction
    sqlx::query!("DELETE FROM transactions WHERE id = ?", transaction_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Calculate user debts for a group (combines shared debts and transactions)
#[server(CalculateUserDebts)]
pub async fn calculate_user_debts(group_id: i64) -> Result<Vec<UserBalance>, ServerFnError> {
    use sqlx::SqlitePool;
    use std::collections::HashMap;

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
        let user_debts = debts.get(&member.id).unwrap();

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
            *debts
                .get_mut(&user_id)
                .unwrap()
                .entry(creator_id)
                .or_insert(Decimal::ZERO) += share_per_user;

            // Creator is owed by user (negative debt)
            *debts
                .get_mut(&creator_id)
                .unwrap()
                .entry(user_id)
                .or_insert(Decimal::ZERO) -= share_per_user;
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
        *debts
            .get_mut(&payer_id)
            .unwrap()
            .entry(recipient_id)
            .or_insert(Decimal::ZERO) -= amount;

        // Recipient is owed less by payer (or owes more to payer)
        *debts
            .get_mut(&recipient_id)
            .unwrap()
            .entry(payer_id)
            .or_insert(Decimal::ZERO) += amount;
    }

    Ok(())
}
