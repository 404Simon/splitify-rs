use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;
use time::OffsetDateTime;

/// Invite model representing an invite link to a group
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Invite {
    pub uuid: String,
    pub group_id: i64,
    pub name: Option<String>,
    pub is_reusable: bool,
    pub duration_days: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Invite with group information for display
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InviteWithGroup {
    pub uuid: String,
    pub group_id: i64,
    pub group_name: String,
    pub name: Option<String>,
    pub is_reusable: bool,
    pub duration_days: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub is_valid: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

/// Invite list item with formatted expiration date
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InviteListItem {
    pub uuid: String,
    pub name: Option<String>,
    pub is_reusable: bool,
    pub expiration_date: String,
}
