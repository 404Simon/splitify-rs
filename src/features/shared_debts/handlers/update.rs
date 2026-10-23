use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Update a shared debt
#[server(UpdateSharedDebt)]
pub async fn update_shared_debt(
    debt_id: i64,
    name: String,
    amount: String,
    member_ids: Vec<i64>,
) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated. Please log in."))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::shared_debts::service::update_debt(
        &pool, user.id, debt_id, name, amount, member_ids,
    )
    .await
}
