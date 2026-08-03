//! Authorization helpers shared by features that operate on groups.

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

/// Ensure `user_id` is a member of `group_id`.
#[cfg(feature = "ssr")]
pub async fn verify_group_membership(
    pool: &SqlitePool,
    user_id: i64,
    group_id: i64,
) -> Result<(), ServerFnError> {
    let is_member = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM group_members
            WHERE group_id = ? AND user_id = ?
        ) as "exists!"
        "#,
        group_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_member == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Not a member of this group".to_string(),
        ));
    }

    Ok(())
}
