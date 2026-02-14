use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;
use time::{Date, OffsetDateTime};

/// Frequency enum for recurring debts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Frequency {
    pub fn as_str(&self) -> &str {
        match self {
            Frequency::Daily => "daily",
            Frequency::Weekly => "weekly",
            Frequency::Monthly => "monthly",
            Frequency::Yearly => "yearly",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Frequency::Daily => "Daily",
            Frequency::Weekly => "Weekly",
            Frequency::Monthly => "Monthly",
            Frequency::Yearly => "Yearly",
        }
    }
}

impl std::str::FromStr for Frequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(Frequency::Daily),
            "weekly" => Ok(Frequency::Weekly),
            "monthly" => Ok(Frequency::Monthly),
            "yearly" => Ok(Frequency::Yearly),
            _ => Err(format!("Invalid frequency: {}", s)),
        }
    }
}

impl std::fmt::Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// RecurringDebt model representing a recurring debt schedule
/// Note: We don't derive FromRow because amount needs custom parsing from TEXT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecurringDebt {
    pub id: i64,
    pub group_id: i64,
    pub created_by: i64,
    pub name: String,
    pub amount: Decimal,
    pub frequency: Frequency,
    pub start_date: Date,
    pub end_date: Option<Date>,
    pub next_generation_date: Date,
    pub is_active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// RecurringDebt with additional information for display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecurringDebtWithDetails {
    pub id: i64,
    pub group_id: i64,
    pub created_by: i64,
    pub creator_username: String,
    pub name: String,
    pub amount: Decimal,
    pub frequency: Frequency,
    pub start_date: Date,
    pub end_date: Option<Date>,
    pub next_generation_date: Date,
    pub is_active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub is_creator: bool,
    pub status: String, // "Active", "Paused", or "Expired"
}

/// Pivot table entry for recurring_debt_user
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct RecurringDebtUser {
    pub id: i64,
    pub recurring_debt_id: i64,
    pub user_id: i64,
}

/// Member information for a recurring debt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecurringDebtMember {
    pub user_id: i64,
    pub username: String,
    pub share_amount: Decimal,
}

/// Generated instance linking a SharedDebt to its RecurringDebt parent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedInstance {
    pub shared_debt_id: i64,
    pub debt_name: String,
    pub amount: Decimal,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
