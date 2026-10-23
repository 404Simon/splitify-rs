use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use super::models::{MapConfig, MapMarker, PlaceSearchResult};
#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;

/// Server function: Get all markers on a group's map.
#[server(GetGroupMapMarkers)]
pub async fn get_group_map_markers(group_id: i64) -> Result<Vec<MapMarker>, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::maps::service::list_markers(&pool, user.id, group_id).await
}

/// Server function: Add a marker to a group's map.
#[server(CreateMapMarker)]
pub async fn create_map_marker(
    group_id: i64,
    name: String,
    description: Option<String>,
    address: Option<String>,
    emoji: String,
    latitude: f64,
    longitude: f64,
) -> Result<i64, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::maps::service::create_marker(
        &pool,
        user.id,
        group_id,
        name,
        description,
        address,
        emoji,
        latitude,
        longitude,
    )
    .await
}

/// Server function: Update a marker on a group's map.
#[server(UpdateMapMarker)]
pub async fn update_map_marker(
    marker_id: i64,
    name: String,
    description: Option<String>,
    address: Option<String>,
    emoji: String,
    latitude: f64,
    longitude: f64,
) -> Result<(), ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::maps::service::update_marker(
        &pool,
        user.id,
        marker_id,
        name,
        description,
        address,
        emoji,
        latitude,
        longitude,
    )
    .await
}

/// Server function: Delete a marker from a group's map.
#[server(DeleteMapMarker)]
pub async fn delete_map_marker(marker_id: i64) -> Result<(), ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    crate::features::maps::service::delete_marker(&pool, user.id, marker_id).await
}

/// Server function: Get the map configuration for the client.
///
/// All values come from environment variables so they can be changed without
/// rebuilding the client bundle.
#[server(GetMapConfig)]
pub async fn get_map_config() -> Result<MapConfig, ServerFnError> {
    crate::features::maps::service::map_config()
}

/// Server function: Search for an address or place using the Nominatim
/// geocoding API (same endpoint as the legacy Splitify app).
///
/// Nominatim's public instance requires a descriptive `User-Agent`; configure
/// it via the `NOMINATIM_USER_AGENT` environment variable (see the legacy
/// `GeolocationService` for reference). The base URL can be overridden with
/// `NOMINATIM_URL` if you self-host Nominatim.
#[server(SearchPlaces)]
pub async fn search_places(query: String) -> Result<Vec<PlaceSearchResult>, ServerFnError> {
    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    crate::features::maps::service::search_places(query).await
}
