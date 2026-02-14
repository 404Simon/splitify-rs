use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::FromRow;
use time::OffsetDateTime;

/// Transaction record from database
#[cfg_attr(feature = "ssr", derive(FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub group_id: i64,
    pub payer_id: i64,
    pub recipient_id: i64,
    // Note: amount is stored as TEXT in SQLite, so we don't use FromRow
    // and will parse manually in queries
    #[cfg_attr(feature = "ssr", sqlx(skip))]
    pub amount: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Transaction with user details (payer and recipient names)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionWithDetails {
    pub id: i64,
    pub group_id: i64,
    pub payer_id: i64,
    pub payer_username: String,
    pub recipient_id: i64,
    pub recipient_username: String,
    pub amount: String, // Stored as string to maintain precision
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// User balance information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserBalance {
    pub user_id: i64,
    pub username: String,
    pub relationships: Vec<DebtRelationship>,
    pub total_owed: String,  // Amount others owe to this user
    pub total_owing: String, // Amount this user owes to others
    pub net_amount: String,  // Absolute value of net balance
    pub net_type: NetType,   // Whether user is net positive, negative, or neutral
}

/// Relationship between two users (one owes the other)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebtRelationship {
    pub other_user_id: i64,
    pub other_username: String,
    pub amount: String,
    pub relationship_type: RelationshipType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    Owes, // Current user owes money to other_user
    Owed, // Other user owes money to current user
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NetType {
    Positive, // User is owed more than they owe
    Negative, // User owes more than they are owed
    Neutral,  // User is balanced
}
