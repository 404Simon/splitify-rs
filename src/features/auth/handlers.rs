use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use super::models::User;
use super::models::UserSession;
#[cfg(feature = "ssr")]
use super::utils::{
    clear_session, get_user_from_session, hash_password, set_user_in_session, verify_password,
};
#[cfg(feature = "ssr")]
use crate::validation::{validate_email, validate_password, validate_username};

/// Server function: Register a new user
#[server(RegisterUser)]
pub async fn register_user(
    username: String,
    password: String,
    email: Option<String>,
) -> Result<UserSession, ServerFnError> {
    use sqlx::SqlitePool;

    // Validate username
    let username = validate_username(&username)?;

    // Validate password
    validate_password(&password)?;

    // Validate email if provided
    let email = match email {
        Some(ref email_str) if !email_str.trim().is_empty() => Some(validate_email(email_str)?),
        _ => None,
    };

    // Get pool from context (provided in main.rs)
    let pool = expect_context::<SqlitePool>();
    let password_hash = hash_password(&password).map_err(|e| ServerFnError::new(e.to_string()))?;

    // Check if user already exists
    let existing = sqlx::query!("SELECT id FROM users WHERE username = ?", username)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if existing.is_some() {
        return Err(ServerFnError::new(
            "Username already exists. Please choose a different username.",
        ));
    }

    // Insert new user
    let result = sqlx::query!(
        "INSERT INTO users (username, password_hash, email) VALUES (?, ?, ?)",
        username,
        password_hash,
        email
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_id = result.last_insert_rowid();

    // Extract session using Axum extractor pattern
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user_session = UserSession {
        id: user_id,
        username: username.clone(),
    };

    set_user_in_session(&session, &user_session)
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    Ok(user_session)
}

/// Server function: Login an existing user
#[server(LoginUser)]
pub async fn login_user(username: String, password: String) -> Result<UserSession, ServerFnError> {
    use sqlx::SqlitePool;

    use crate::validation::sanitize_string;

    // Basic validation (don't validate username format on login, only sanitize)
    let username = sanitize_string(&username);
    if username.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Username and password are required"));
    }

    // Get pool from context (provided in main.rs)
    let pool = expect_context::<SqlitePool>();

    // Fetch user from database
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, email FROM users WHERE username = ?",
    )
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Use constant-time comparison to prevent timing attacks
    // Always verify password even if user doesn't exist
    let (valid, user_session) = match user {
        Some(user) => {
            let valid = verify_password(&password, &user.password_hash).unwrap_or(false);

            let session = UserSession {
                id: user.id,
                username: user.username,
            };
            (valid, Some(session))
        }
        None => {
            // Still perform password hash verification to maintain constant timing
            // Use a dummy hash to prevent timing differences
            let _ = verify_password(
                &password,
                "$2b$12$dummy.hash.to.prevent.timing.attacks.abcdefghijklmnopqr",
            );
            (false, None)
        }
    };

    if !valid {
        return Err(ServerFnError::new("Invalid username or password"));
    }

    let user_session = user_session.ok_or_else(|| ServerFnError::new("Authentication error"))?;

    // Extract session using Axum extractor pattern
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    set_user_in_session(&session, &user_session)
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    Ok(user_session)
}

/// Server function: Logout the current user
#[server(LogoutUser)]
pub async fn logout_user() -> Result<(), ServerFnError> {
    // Extract session using Axum extractor pattern
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    clear_session(&session)
        .await
        .map_err(|_| ServerFnError::new("Logout failed"))?;
    Ok(())
}

/// Server function: Get the current user session
#[server(GetUser)]
pub async fn get_user() -> Result<Option<UserSession>, ServerFnError> {
    // Extract session using Axum extractor pattern
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    Ok(get_user_from_session(&session).await)
}
