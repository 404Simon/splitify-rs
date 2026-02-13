use super::models::UserSession;

#[cfg(feature = "ssr")]
use bcrypt::{hash, verify, DEFAULT_COST};
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

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

/// Custom hook for handling logout with automatic navigation and context refresh
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
/// ```
/// use crate::features::auth::use_logout;
///
/// #[component]
/// pub fn MyComponent() -> impl IntoView {
///     let on_logout = use_logout();
///     
///     view! {
///         <button on:click=move |_| on_logout.run(())>
///             "Logout"
///         </button>
///     }
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
