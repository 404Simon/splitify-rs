//! Create operations for transactions

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Create a new transaction
#[server(CreateTransaction)]
pub async fn create_transaction(
    group_id: i64,
    recipient_id: i64,
    amount: String,
    description: Option<String>,
) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::transactions::service::create_transaction(
        &pool,
        user.id,
        group_id,
        recipient_id,
        amount,
        description,
    )
    .await
}
