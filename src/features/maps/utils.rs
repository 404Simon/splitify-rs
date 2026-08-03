#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

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

#[cfg(feature = "ssr")]
pub async fn verify_marker_creator(
    pool: &SqlitePool,
    user_id: i64,
    marker_id: i64,
) -> Result<(), ServerFnError> {
    let is_creator = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM group_map_markers
            WHERE id = ? AND created_by = ?
        ) as "exists!"
        "#,
        marker_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_creator == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Only the marker creator can perform this action".to_string(),
        ));
    }

    Ok(())
}

/// Validate a coordinate pair. Latitude must be in [-90, 90] and longitude in
/// [-180, 180]; both must be finite numbers.
#[cfg(feature = "ssr")]
pub fn validate_coordinates(latitude: f64, longitude: f64) -> Result<(), ServerFnError> {
    if !latitude.is_finite() || !longitude.is_finite() {
        return Err(ServerFnError::new(
            "Coordinates must be valid numbers".to_string(),
        ));
    }
    if !(-90.0..=90.0).contains(&latitude) {
        return Err(ServerFnError::new(format!(
            "Latitude must be between -90 and 90 (got {latitude})"
        )));
    }
    if !(-180.0..=180.0).contains(&longitude) {
        return Err(ServerFnError::new(format!(
            "Longitude must be between -180 and 180 (got {longitude})"
        )));
    }
    Ok(())
}

/// Normalize an optional description: empty/whitespace-only becomes `None`.
#[cfg(feature = "ssr")]
pub fn normalize_description(description: Option<String>) -> Option<String> {
    description
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
}

/// Normalize an optional address: empty/whitespace-only becomes `None`.
#[cfg(feature = "ssr")]
pub fn normalize_address(address: Option<String>) -> Option<String> {
    normalize_description(address)
}
