use super::models::MapMarker;

#[cfg(feature = "ssr")]
use super::models::DEFAULT_MARKER_EMOJI;
#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

/// Whether the user can manage a marker: the creator or a group admin.
///
/// Mirrors the server-side authorization in [`verify_marker_creator`], so the
/// UI never shows actions the server will reject.
pub fn can_manage_marker(marker: &MapMarker, user_id: i64, is_admin: bool) -> bool {
    marker.created_by == user_id || is_admin
}

/// Google Maps "directions" URL for a marker, matching the legacy Splitify
/// behavior (`google.com/maps/dir/?api=1&destination=...`). Opening it on a
/// phone hands off straight to the Google Maps app. Prefers the saved address,
/// falling back to the exact coordinates.
pub fn google_maps_nav_url(marker: &MapMarker) -> String {
    let destination = marker
        .address
        .as_deref()
        .map(str::trim)
        .filter(|address| !address.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{},{}", marker.latitude, marker.longitude));
    format!(
        "https://www.google.com/maps/dir/?api=1&destination={}",
        urlencoding::encode(&destination)
    )
}

/// Format a coordinate for display.
pub fn format_coordinate(value: f64) -> String {
    format!("{value:.5}")
}

/// Ensure the user is the marker's creator or a group admin, mirroring the
/// [`can_manage_marker`] UI check on the server.
#[cfg(feature = "ssr")]
pub async fn verify_marker_creator(
    pool: &SqlitePool,
    user_id: i64,
    marker_id: i64,
) -> Result<(), ServerFnError> {
    let is_manager = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM group_map_markers m
            WHERE m.id = ?
              AND (m.created_by = ?
                   OR (SELECT created_by FROM groups WHERE id = m.group_id) = ?)
        ) as "exists!"
        "#,
        marker_id,
        user_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if is_manager == 0 {
        return Err(ServerFnError::new(
            "Unauthorized: Only the marker creator or a group admin can perform this action"
                .to_string(),
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

/// The marker icon shown on the map. Falls back to the default pin when
/// missing, and is capped so it cannot be abused.
#[cfg(feature = "ssr")]
pub fn normalize_emoji(emoji: Option<String>) -> String {
    emoji
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.chars().count() <= 8 {
                value
            } else {
                DEFAULT_MARKER_EMOJI.to_string()
            }
        })
        .unwrap_or_else(|| DEFAULT_MARKER_EMOJI.to_string())
}
