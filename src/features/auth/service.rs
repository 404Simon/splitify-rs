#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

#[cfg(feature = "ssr")]
use super::models::{User, UserSession};
#[cfg(feature = "ssr")]
use super::utils::{hash_password, verify_password};
#[cfg(feature = "ssr")]
use crate::validation::{validate_email, validate_password, validate_username};

/// Validate and create a new user account.
#[cfg(feature = "ssr")]
pub async fn register_user(
    pool: &SqlitePool,
    username: String,
    password: String,
    email: Option<String>,
) -> Result<UserSession, ServerFnError> {
    let username = validate_username(&username)?;

    validate_password(&password)?;

    let email = match email {
        Some(ref email_str) if !email_str.trim().is_empty() => Some(validate_email(email_str)?),
        _ => None,
    };

    let password_hash = hash_password(&password).map_err(|e| ServerFnError::new(e.to_string()))?;

    let existing = sqlx::query!("SELECT id FROM users WHERE username = ?", username)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if existing.is_some() {
        return Err(ServerFnError::new(
            "Username already exists. Please choose a different username.",
        ));
    }

    let result = sqlx::query!(
        "INSERT INTO users (username, password_hash, email) VALUES (?, ?, ?)",
        username,
        password_hash,
        email
    )
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(UserSession {
        id: result.last_insert_rowid(),
        username,
    })
}

/// Verify credentials and return the matching user.
#[cfg(feature = "ssr")]
pub async fn authenticate_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<UserSession, ServerFnError> {
    use crate::validation::sanitize_string;

    let username = sanitize_string(username);
    if username.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Username and password are required"));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email FROM users WHERE username = ?",
    )
    .bind(&username)
    .fetch_optional(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let (valid, user_session) = match user {
        Some(user) => {
            let valid = verify_password(password, &user.password_hash).unwrap_or(false);

            let session = UserSession {
                id: user.id,
                username: user.username,
            };
            (valid, Some(session))
        }
        None => {
            let _ = verify_password(
                password,
                "$2b$12$dummy.hash.to.prevent.timing.attacks.abcdefghijklmnopqr",
            );
            (false, None)
        }
    };

    if !valid {
        return Err(ServerFnError::new("Invalid username or password"));
    }

    user_session.ok_or_else(|| ServerFnError::new("Authentication error"))
}
