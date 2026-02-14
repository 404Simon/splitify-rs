use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[cfg(feature = "ssr")]
use sqlx::FromRow;

/// SharedDebt model representing a shared expense in a group
/// Note: We don't derive FromRow because amount needs custom parsing from TEXT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedDebt {
    pub id: i64,
    pub group_id: i64,
    pub created_by: i64,
    pub name: String,
    pub amount: Decimal,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// SharedDebt with additional information for display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedDebtWithDetails {
    pub id: i64,
    pub group_id: i64,
    pub created_by: i64,
    pub creator_username: String,
    pub name: String,
    pub amount: Decimal,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub is_creator: bool,
}

/// User share information for a shared debt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserShare {
    pub user_id: i64,
    pub username: String,
    pub share_amount: Decimal,
}

/// Pivot table entry for shared_debt_user
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct SharedDebtUser {
    pub id: i64,
    pub shared_debt_id: i64,
    pub user_id: i64,
}
