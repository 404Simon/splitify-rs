use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
use crate::features::shared_debts::models::{SharedDebtWithDetails, UserShare};

/// Server function: Get a specific shared debt
#[server(GetSharedDebt)]
pub async fn get_shared_debt(debt_id: i64) -> Result<SharedDebtWithDetails, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::shared_debts::service::get_debt(&pool, user.id, debt_id).await
}

/// Server function: Get all shared debts for a group
#[server(GetGroupSharedDebts)]
pub async fn get_group_shared_debts(
    group_id: i64,
) -> Result<Vec<SharedDebtWithDetails>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::shared_debts::service::list_group_debts(&pool, user.id, group_id).await
}

/// Server function: Get user IDs involved in a shared debt
#[server(GetSharedDebtMembers)]
pub async fn get_shared_debt_members(debt_id: i64) -> Result<Vec<i64>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::shared_debts::service::list_debt_members(&pool, user.id, debt_id).await
}

/// Server function: Get user shares for a specific shared debt
#[server(GetSharedDebtShares)]
pub async fn get_shared_debt_shares(debt_id: i64) -> Result<Vec<UserShare>, ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::shared_debts::service::list_debt_shares(&pool, user.id, debt_id).await
}
