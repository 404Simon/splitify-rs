use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::extract;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;
#[cfg(feature = "ssr")]
use tower_sessions::Session;

use super::models::{MapConfig, MapMarker, PlaceSearchResult};
#[cfg(feature = "ssr")]
use super::utils::{
    normalize_address, normalize_description, normalize_emoji, validate_coordinates,
    verify_marker_creator,
};
#[cfg(feature = "ssr")]
use crate::features::auth::utils::get_user_from_session;
#[cfg(feature = "ssr")]
use crate::features::groups::permissions::verify_group_membership;
#[cfg(feature = "ssr")]
use crate::validation::{validate_description, validate_name};

/// Sanitized marker fields, ready for persistence. Produced by
/// [`validate_marker_input`] so create and update share identical validation.
#[cfg(feature = "ssr")]
struct MarkerInput {
    name: String,
    description: Option<String>,
    address: Option<String>,
    emoji: String,
    latitude: f64,
    longitude: f64,
}

/// Validate the fields shared by marker create and update.
#[cfg(feature = "ssr")]
fn validate_marker_input(
    name: String,
    description: Option<String>,
    address: Option<String>,
    emoji: String,
    latitude: f64,
    longitude: f64,
) -> Result<MarkerInput, ServerFnError> {
    let name = validate_name(&name, 1, 255, "Marker name")?;
    let description = validate_description(&description.unwrap_or_default(), 500)?;
    let description = normalize_description(Some(description));
    let address = normalize_address(address);
    let emoji = normalize_emoji(Some(emoji));
    validate_coordinates(latitude, longitude)?;

    Ok(MarkerInput {
        name,
        description,
        address,
        emoji,
        latitude,
        longitude,
    })
}

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

    verify_group_membership(&pool, user.id, group_id).await?;

    let rows = sqlx::query!(
        r#"
        SELECT
            m.id as "id!",
            m.group_id as "group_id!",
            m.created_by as "created_by!",
            u.username as creator_username,
            m.name,
            m.description,
            m.address,
            m.emoji,
            m.latitude as "latitude!",
            m.longitude as "longitude!",
            m.created_at,
            m.updated_at
        FROM group_map_markers m
        INNER JOIN users u ON m.created_by = u.id
        WHERE m.group_id = ?
        ORDER BY m.created_at ASC
        "#,
        group_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|row| MapMarker {
            id: row.id,
            group_id: row.group_id,
            created_by: row.created_by,
            creator_username: row.creator_username,
            name: row.name,
            description: row.description,
            address: row.address,
            emoji: row.emoji,
            latitude: row.latitude,
            longitude: row.longitude,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect())
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
    let input = validate_marker_input(name, description, address, emoji, latitude, longitude)?;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    verify_group_membership(&pool, user.id, group_id).await?;

    let result = sqlx::query!(
        r#"
        INSERT INTO group_map_markers (group_id, created_by, name, description, address, emoji, latitude, longitude)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        group_id,
        user.id,
        input.name,
        input.description,
        input.address,
        input.emoji,
        input.latitude,
        input.longitude
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
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
    let input = validate_marker_input(name, description, address, emoji, latitude, longitude)?;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    let user = get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let pool = expect_context::<SqlitePool>();

    verify_marker_creator(&pool, user.id, marker_id).await?;

    sqlx::query!(
        r#"
        UPDATE group_map_markers
        SET name = ?, description = ?, address = ?, emoji = ?, latitude = ?, longitude = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
        input.name,
        input.description,
        input.address,
        input.emoji,
        input.latitude,
        input.longitude,
        marker_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
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

    verify_marker_creator(&pool, user.id, marker_id).await?;

    sqlx::query!("DELETE FROM group_map_markers WHERE id = ?", marker_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Server function: Get the map configuration for the client.
///
/// All values come from environment variables so they can be changed without
/// rebuilding the client bundle.
#[server(GetMapConfig)]
pub async fn get_map_config() -> Result<MapConfig, ServerFnError> {
    let style_url = std::env::var("MAP_STYLE_URL")
        .unwrap_or_else(|_| "https://tiles.openfreemap.org/styles/liberty".to_string());
    let dark_style_url = std::env::var("MAP_DARK_STYLE_URL")
        .unwrap_or_else(|_| "https://tiles.openfreemap.org/styles/dark".to_string());

    let default_lng = std::env::var("MAP_DEFAULT_LNG")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(13.405);
    let default_lat = std::env::var("MAP_DEFAULT_LAT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(52.52);
    let default_zoom = std::env::var("MAP_DEFAULT_ZOOM")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(10.0);

    Ok(MapConfig {
        style_url,
        dark_style_url,
        default_lng,
        default_lat,
        default_zoom,
    })
}

/// The shared HTTP client used for geocoding. Built once to avoid recreating
/// the connection pool on every search.
#[cfg(feature = "ssr")]
fn geocoding_client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        let user_agent = std::env::var("NOMINATIM_USER_AGENT")
            .unwrap_or_else(|_| "Splitify/0.2 (expense sharing app)".to_string());
        reqwest::Client::builder()
            .user_agent(user_agent)
            .build()
            .expect("failed to build the geocoding HTTP client")
    })
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
    use serde::Deserialize;

    let session = extract::<Session>()
        .await
        .map_err(|_| ServerFnError::new("Authentication error"))?;
    get_user_from_session(&session)
        .await
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    let query = query.trim();
    if query.is_empty() {
        return Err(ServerFnError::new("Search query cannot be empty"));
    }
    if query.len() > 200 {
        return Err(ServerFnError::new("Search query is too long"));
    }

    let base_url = std::env::var("NOMINATIM_URL")
        .unwrap_or_else(|_| "https://nominatim.openstreetmap.org/search".to_string());

    #[derive(Deserialize)]
    struct RawResult {
        display_name: String,
        lat: String,
        lon: String,
    }

    let results = geocoding_client()
        .get(base_url)
        .query(&[("q", query), ("format", "json"), ("limit", "5")])
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Geocoding request failed: {e}")))?
        .json::<Vec<RawResult>>()
        .await
        .map_err(|e| ServerFnError::new(format!("Geocoding response invalid: {e}")))?;

    Ok(results
        .into_iter()
        .filter_map(|result| {
            let lat = result.lat.parse::<f64>().ok()?;
            let lon = result.lon.parse::<f64>().ok()?;
            Some(PlaceSearchResult {
                display_name: result.display_name,
                lat,
                lon,
            })
        })
        .collect())
}
