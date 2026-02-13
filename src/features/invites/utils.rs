#[cfg(feature = "ssr")]
use time::{Duration, OffsetDateTime};

#[cfg(feature = "ssr")]
use uuid::Uuid;

/// Generate a new UUID v4 for an invite
#[cfg(feature = "ssr")]
pub fn generate_invite_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Check if an invite is valid based on expiration
#[cfg(feature = "ssr")]
pub fn is_invite_valid(created_at: &OffsetDateTime, duration_days: i64) -> bool {
    if duration_days <= 0 {
        return false;
    }

    let expiration = *created_at + Duration::days(duration_days);
    OffsetDateTime::now_utc() < expiration
}

/// Calculate expiration date for an invite
#[cfg(feature = "ssr")]
pub fn calculate_expiration(created_at: &OffsetDateTime, duration_days: i64) -> OffsetDateTime {
    *created_at + Duration::days(duration_days)
}
