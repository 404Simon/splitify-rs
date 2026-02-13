#[cfg(feature = "ssr")]
use super::models::UserSession;
#[cfg(feature = "ssr")]
use bcrypt::{hash, verify, DEFAULT_COST};
#[cfg(feature = "ssr")]
use tower_sessions::Session;

/// Hash a password using bcrypt with default cost
#[cfg(feature = "ssr")]
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

/// Verify a password against a bcrypt hash
#[cfg(feature = "ssr")]
pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

/// Validate email format using simple regex pattern
#[cfg(feature = "ssr")]
pub fn is_valid_email(email: &str) -> bool {
    // Simple email validation - checks for basic format: something@something.something
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part should not be empty and domain should contain a dot
    !local.is_empty() && domain.contains('.') && domain.len() > 3
}

/// Retrieve user session from tower-sessions
#[cfg(feature = "ssr")]
pub async fn get_user_from_session(session: &Session) -> Option<UserSession> {
    session.get::<UserSession>("user").await.ok().flatten()
}

/// Store user session in tower-sessions
#[cfg(feature = "ssr")]
pub async fn set_user_in_session(
    session: &Session,
    user: &UserSession,
) -> Result<(), tower_sessions::session::Error> {
    session.insert("user", user).await
}

/// Clear the current session (logout)
#[cfg(feature = "ssr")]
pub async fn clear_session(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.delete().await
}
