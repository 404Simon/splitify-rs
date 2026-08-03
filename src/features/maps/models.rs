use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Default marker icon used when no emoji is chosen.
pub const DEFAULT_MARKER_EMOJI: &str = "📍";

/// A location marker placed by a group member on the group's map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMarker {
    pub id: i64,
    pub group_id: i64,
    pub created_by: i64,
    pub creator_username: String,
    pub name: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub emoji: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// A single geocoding result from the address/place search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceSearchResult {
    pub display_name: String,
    pub lat: f64,
    pub lon: f64,
}

/// One emoji in the picker (see `src/js/emoji.json`).
#[derive(Debug, Clone, Deserialize)]
pub struct EmojiEntry {
    pub emoji: String,
    pub title: String,
}

/// A named category of emojis for the picker.
#[derive(Debug, Clone)]
pub struct EmojiCategory {
    pub name: String,
    pub emojis: Vec<EmojiEntry>,
}

/// Client-facing map configuration resolved on the server so that the map
/// style and initial view can be tuned without a client rebuild.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    pub style_url: String,
    pub dark_style_url: String,
    pub default_lng: f64,
    pub default_lat: f64,
    pub default_zoom: f64,
}

/// Imperative commands the page sends to the rendered map.
#[derive(Debug, Clone, Copy)]
pub enum MapCommand {
    /// Fit the viewport to the current set of markers.
    Fit,
    /// Smoothly fly to a coordinate.
    FlyTo { lng: f64, lat: f64 },
    /// Gently pan to a coordinate without a zoom animation.
    CenterOn { lng: f64, lat: f64 },
}
