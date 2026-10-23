use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use super::models::UserSession;
#[cfg(feature = "ssr")]
use super::utils::{clear_session, get_user_from_session, set_user_in_session};

/// Server function: Register a new user
#[server(RegisterUser)]
pub async fn register_user(
    username: String,
    password: String,
    email: Option<String>,
) -> Result<UserSession, ServerFnError> {
    use sqlx::SqlitePool;

    let pool = expect_context::<SqlitePool>();
    let user_session = super::service::register_user(&pool, username, password, email).await?;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    set_user_in_session(&session, &user_session)
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    Ok(user_session)
}

/// Server function: Login an existing user
#[server(LoginUser)]
pub async fn login_user(username: String, password: String) -> Result<UserSession, ServerFnError> {
    use sqlx::SqlitePool;

    let pool = expect_context::<SqlitePool>();
    let user_session = super::service::authenticate_user(&pool, &username, &password).await?;

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
