//! Operations for managing generated instances (SharedDebts)

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
use crate::features::recurring_debts::models::GeneratedInstance;

/// Server function: Get generated instances (SharedDebts) from a recurring debt
#[server(GetGeneratedInstances)]
pub async fn get_generated_instances(
    recurring_debt_id: i64,
) -> Result<Vec<GeneratedInstance>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::list_instances(&pool, user.id, recurring_debt_id)
        .await
}

/// Server function: Manually generate a shared debt from a recurring debt
#[server(GenerateNow)]
pub async fn generate_now(recurring_debt_id: i64) -> Result<i64, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::recurring_debts::service::generate_now(&pool, user.id, recurring_debt_id).await
}
