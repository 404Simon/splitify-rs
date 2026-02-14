use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Delete a shared debt
#[server(DeleteSharedDebt)]
pub async fn delete_shared_debt(debt_id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;

    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    // Check if user is the creator of the debt
    let debt = sqlx::query!("SELECT created_by FROM shared_debts WHERE id = ?", debt_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Shared debt not found"))?;

    if debt.created_by != user.id {
        return Err(ServerFnError::new(
            "Unauthorized: Only the creator can delete this debt",
        ));
    }

    // Delete the debt (CASCADE will handle related data)
    sqlx::query!("DELETE FROM shared_debts WHERE id = ?", debt_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
