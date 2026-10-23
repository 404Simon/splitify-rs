//! Whole-group snapshot used by the offline-first Android client.
//!
//! A single call returns everything needed to rebuild a group locally: the
//! group, its members, debts with their splits, transactions, shopping lists
//! with items and activity, recurring debts with members/shares/instances,
//! map markers and invites. Balances are intentionally omitted; clients derive
//! them from debts and transactions.

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use sqlx::SqlitePool;

use super::{error::ApiError, extractors::ApiUser};
use crate::features::{
    groups::models::{Group, GroupMemberInfo},
    invites::models::InviteListItem,
    maps::models::MapMarker,
    recurring_debts::models::{GeneratedInstance, RecurringDebtMember, RecurringDebtWithDetails},
    shared_debts::models::{SharedDebtWithDetails, UserShare},
    shopping_lists::models::{ShoppingList, ShoppingListActivity, ShoppingListItem},
    transactions::models::TransactionWithDetails,
};

/// Number of activity entries returned per shopping list.
const ACTIVITY_LIMIT: i64 = 10;

/// Complete state of one group.
#[derive(Debug, Serialize)]
pub struct GroupSnapshot {
    pub group: Group,
    pub members: Vec<GroupMemberInfo>,
    pub debts: Vec<DebtSnapshot>,
    pub transactions: Vec<TransactionWithDetails>,
    pub shopping_lists: Vec<ShoppingListSnapshot>,
    pub recurring_debts: Vec<RecurringDebtSnapshot>,
    pub markers: Vec<MapMarker>,
    pub invites: Vec<InviteListItem>,
    pub generated_at: String,
}

/// A shared debt with its participant ids and computed equal shares.
#[derive(Debug, Serialize)]
pub struct DebtSnapshot {
    pub debt: SharedDebtWithDetails,
    pub participant_ids: Vec<i64>,
    pub shares: Vec<UserShare>,
}

/// A shopping list with its items and most recent activity.
#[derive(Debug, Serialize)]
pub struct ShoppingListSnapshot {
    pub list: ShoppingList,
    pub items: Vec<ShoppingListItem>,
    pub activity: Vec<ShoppingListActivity>,
}

/// A recurring debt with its members, shares and generated instances.
#[derive(Debug, Serialize)]
pub struct RecurringDebtSnapshot {
    pub debt: RecurringDebtWithDetails,
    pub member_ids: Vec<i64>,
    pub shares: Vec<RecurringDebtMember>,
    pub instances: Vec<GeneratedInstance>,
}

/// `GET /api/v1/groups/{group_id}/snapshot`
pub async fn group_snapshot(
    Path(group_id): Path<i64>,
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<GroupSnapshot>, ApiError> {
    let group = crate::features::groups::service::get_group(&pool, user.id, group_id).await?;

    let members =
        crate::features::groups::service::list_group_members(&pool, user.id, group_id).await?;

    let debts =
        crate::features::shared_debts::service::list_group_debts(&pool, user.id, group_id).await?;

    let mut debt_snapshots = Vec::with_capacity(debts.len());
    for debt in debts {
        let participant_ids =
            crate::features::shared_debts::service::list_debt_members(&pool, user.id, debt.id)
                .await?;
        let shares =
            crate::features::shared_debts::service::list_debt_shares(&pool, user.id, debt.id)
                .await?;

        debt_snapshots.push(DebtSnapshot {
            debt,
            participant_ids,
            shares,
        });
    }

    let transactions =
        crate::features::transactions::service::list_group_transactions(&pool, user.id, group_id)
            .await?;

    let lists =
        crate::features::shopping_lists::service::list_shopping_lists(&pool, user.id, group_id)
            .await?;

    let mut shopping_lists = Vec::with_capacity(lists.len());
    for summary in lists {
        let list =
            crate::features::shopping_lists::service::get_shopping_list(&pool, user.id, summary.id)
                .await?;
        let items =
            crate::features::shopping_lists::service::list_items(&pool, user.id, summary.id)
                .await?;
        let activity = crate::features::shopping_lists::service::list_activity(
            &pool,
            user.id,
            summary.id,
            ACTIVITY_LIMIT,
        )
        .await?;

        shopping_lists.push(ShoppingListSnapshot {
            list,
            items,
            activity,
        });
    }

    let recurring =
        crate::features::recurring_debts::service::list_for_group(&pool, user.id, group_id).await?;

    let mut recurring_debts = Vec::with_capacity(recurring.len());
    for debt in recurring {
        let member_ids =
            crate::features::recurring_debts::service::list_members(&pool, user.id, debt.id)
                .await?;
        let shares =
            crate::features::recurring_debts::service::list_shares(&pool, user.id, debt.id).await?;
        let instances =
            crate::features::recurring_debts::service::list_instances(&pool, user.id, debt.id)
                .await?;

        recurring_debts.push(RecurringDebtSnapshot {
            debt,
            member_ids,
            shares,
            instances,
        });
    }

    let markers = crate::features::maps::service::list_markers(&pool, user.id, group_id).await?;

    // Invites are visible to the group creator only; other members get an
    // empty list rather than a failed snapshot.
    let invites = if group.created_by == user.id {
        crate::features::invites::service::list_group_invites(&pool, user.id, group_id).await?
    } else {
        Vec::new()
    };

    let generated_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    Ok(Json(GroupSnapshot {
        group,
        members,
        debts: debt_snapshots,
        transactions,
        shopping_lists,
        recurring_debts,
        markers,
        invites,
        generated_at,
    }))
}
