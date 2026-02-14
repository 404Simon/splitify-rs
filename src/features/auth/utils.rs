#[cfg(feature = "ssr")]
use bcrypt::{hash, verify, DEFAULT_COST};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use super::models::UserSession;

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

// Email validation has been moved to the centralized validation module
// Use crate::validation::is_valid_email or crate::validation::validate_email
// instead

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

/// Require authentication - returns user session or error
/// This is a helper to reduce boilerplate in server functions
#[cfg(feature = "ssr")]
pub async fn require_auth() -> Result<UserSession, ServerFnError> {
    use leptos_axum::extract;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))
}

/// Custom hook for handling logout with automatic navigation and context
/// refresh
///
/// This hook encapsulates all the logout logic including:
/// - Dispatching the logout server action
/// - Refetching the user resource to update global context
/// - Navigating to the login page
///
/// # Returns
/// A callback that can be used in onClick handlers to trigger logout
///
/// # Example
/// ```ignore
/// let on_logout = use_logout();
///
/// view! {
///     <button on:click=move |_| on_logout.run(())>
///         "Logout"
///     </button>
/// }
/// ```
pub fn use_logout() -> Callback<()> {
    use super::handlers::LogoutUser;

    let logout_action = ServerAction::<LogoutUser>::new();
    let navigate = use_navigate();
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();

    // Effect to handle navigation after successful logout
    Effect::new(move |_| {
        if let Some(Ok(())) = logout_action.value().get() {
            // Refetch user resource to update the global user context
            user_resource.refetch();
            // Navigate to login page
            navigate("/login", Default::default());
        }
    });

    // Return callback that dispatches the logout action
    Callback::new(move |_: ()| {
        logout_action.dispatch(LogoutUser {});
    })
}
