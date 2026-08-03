//! Client-side bridge to the bundled MapLibre glue module.
//!
//! This module wraps `window.SplitifyMap` (built from `src/js/map.js` by
//! Rollup) so that the rest of the application never touches raw
//! `js_sys`/`web_sys` calls. It is only compiled into the WASM hydration
//! bundle — never into the server binary.

use js_sys::{Array, Function, Object, Reflect};
use serde::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

use super::models::MapMarker;

/// Closures handed to the JS glue so MapLibre events can reach Rust state.
///
/// The closures must be kept alive for as long as the map exists; store this
/// struct in a `StoredValue` and drop it on cleanup.
pub struct MapCallbacks {
    on_map_load: Closure<dyn FnMut()>,
    on_map_click: Closure<dyn FnMut(JsValue)>,
    on_marker_click: Closure<dyn FnMut(JsValue)>,
    on_temp_marker_moved: Closure<dyn FnMut(JsValue)>,
}

impl MapCallbacks {
    #[allow(clippy::type_complexity)]
    pub fn new<L, C, M, T>(
        on_map_load: L,
        on_map_click: C,
        on_marker_click: M,
        on_temp_marker_moved: T,
    ) -> Self
    where
        L: FnMut() + 'static,
        C: FnMut(JsValue) + 'static,
        M: FnMut(JsValue) + 'static,
        T: FnMut(JsValue) + 'static,
    {
        Self {
            on_map_load: Closure::new(on_map_load),
            on_map_click: Closure::new(on_map_click),
            on_marker_click: Closure::new(on_marker_click),
            on_temp_marker_moved: Closure::new(on_temp_marker_moved),
        }
    }
}

/// Returns `true` when the `window.SplitifyMap` glue module has loaded.
pub fn glue_loaded() -> bool {
    glue().is_some()
}

/// Resolve the glue object (`window.SplitifyMap`), if it exists yet.
fn glue() -> Option<Object> {
    let window = web_sys::window()?;
    let value = Reflect::get(&window, &JsValue::from_str("SplitifyMap"))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())?;
    Some(value.unchecked_into())
}

/// Convenience wrapper around a failed `js_sys` call.
fn js_err(value: JsValue) -> String {
    format!("{value:?}")
}

/// Look up and invoke a method on the glue object.
fn call_method(glue: &Object, method: &str, args: &Array) -> Result<JsValue, String> {
    let func: Function = Reflect::get(glue, &JsValue::from_str(method))
        .map_err(js_err)?
        .dyn_into()
        .map_err(|_| format!("SplitifyMap.{method} is not a function"))?;
    Reflect::apply(&func, glue.as_ref(), args).map_err(js_err)
}

/// Log a message to the browser console.
pub fn log_error(message: &str) {
    web_sys::console::error_1(&JsValue::from_str(message));
}

/// Create the map inside the container element.
pub fn create_map(
    container_id: &str,
    style_url: &str,
    dark_style_url: &str,
    default_lng: f64,
    default_lat: f64,
    default_zoom: f64,
) -> Result<(), String> {
    let glue = glue().ok_or("SplitifyMap is not loaded")?;
    let options = Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("styleUrl"),
        &JsValue::from_str(style_url),
    )
    .map_err(js_err)?;
    Reflect::set(
        &options,
        &JsValue::from_str("darkStyleUrl"),
        &JsValue::from_str(dark_style_url),
    )
    .map_err(js_err)?;
    Reflect::set(
        &options,
        &JsValue::from_str("center"),
        &Array::of2(
            &JsValue::from_f64(default_lng),
            &JsValue::from_f64(default_lat),
        ),
    )
    .map_err(js_err)?;
    Reflect::set(
        &options,
        &JsValue::from_str("zoom"),
        &JsValue::from_f64(default_zoom),
    )
    .map_err(js_err)?;

    call_method(
        &glue,
        "create",
        &Array::of2(&JsValue::from_str(container_id), &options.into()),
    )
    .map(|_| ())
}

/// Attach event callbacks to the map.
pub fn set_callbacks(container_id: &str, callbacks: &MapCallbacks) -> Result<(), String> {
    let glue = glue().ok_or("SplitifyMap is not loaded")?;
    let callbacks_obj = Object::new();
    Reflect::set(
        &callbacks_obj,
        &JsValue::from_str("onMapLoad"),
        callbacks.on_map_load.as_ref(),
    )
    .map_err(js_err)?;
    Reflect::set(
        &callbacks_obj,
        &JsValue::from_str("onMapClick"),
        callbacks.on_map_click.as_ref(),
    )
    .map_err(js_err)?;
    Reflect::set(
        &callbacks_obj,
        &JsValue::from_str("onMarkerClick"),
        callbacks.on_marker_click.as_ref(),
    )
    .map_err(js_err)?;
    Reflect::set(
        &callbacks_obj,
        &JsValue::from_str("onTempMarkerMoved"),
        callbacks.on_temp_marker_moved.as_ref(),
    )
    .map_err(js_err)?;

    call_method(
        &glue,
        "setCallbacks",
        &Array::of2(&JsValue::from_str(container_id), &callbacks_obj.into()),
    )
    .map(|_| ())
}

/// Shape of a marker as expected by the JS glue.
#[derive(Serialize)]
struct JsMarker<'a> {
    id: i64,
    lng: f64,
    lat: f64,
    name: &'a str,
    emoji: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    creator: &'a str,
}

/// Replace all markers on the map.
pub fn set_markers(container_id: &str, markers: &[MapMarker]) -> Result<(), String> {
    let glue = glue().ok_or("SplitifyMap is not loaded")?;
    let dtos: Vec<JsMarker> = markers
        .iter()
        .map(|marker| JsMarker {
            id: marker.id,
            lng: marker.longitude,
            lat: marker.latitude,
            name: &marker.name,
            emoji: &marker.emoji,
            description: marker.description.as_deref(),
            creator: &marker.creator_username,
        })
        .collect();
    let value = serde_wasm_bindgen::to_value(&dtos).map_err(|e| e.to_string())?;

    call_method(
        &glue,
        "setMarkers",
        &Array::of2(&JsValue::from_str(container_id), &value),
    )
    .map(|_| ())
}

/// Enable/disable "pick a location" mode (crosshair cursor).
pub fn set_add_mode(container_id: &str, enabled: bool) {
    if let Some(glue) = glue() {
        let _ = call_method(
            &glue,
            "setAddMode",
            &Array::of2(
                &JsValue::from_str(container_id),
                &JsValue::from_bool(enabled),
            ),
        );
    }
}

/// Fit the viewport to all markers.
pub fn fit_markers(container_id: &str) {
    if let Some(glue) = glue() {
        let _ = call_method(
            &glue,
            "fitMarkers",
            &Array::of1(&JsValue::from_str(container_id)),
        );
    }
}

/// Smoothly fly the camera to a coordinate.
pub fn fly_to(container_id: &str, lng: f64, lat: f64) {
    if let Some(glue) = glue() {
        let _ = call_method(
            &glue,
            "flyTo",
            &Array::of3(
                &JsValue::from_str(container_id),
                &JsValue::from_f64(lng),
                &JsValue::from_f64(lat),
            ),
        );
    }
}

/// Gently pan the camera to a coordinate without a zoom animation.
pub fn center_on(container_id: &str, lng: f64, lat: f64) {
    if let Some(glue) = glue() {
        let _ = call_method(
            &glue,
            "centerOn",
            &Array::of3(
                &JsValue::from_str(container_id),
                &JsValue::from_f64(lng),
                &JsValue::from_f64(lat),
            ),
        );
    }
}

/// Tear down the map and release its resources.
pub fn destroy_map(container_id: &str) {
    if let Some(glue) = glue() {
        let _ = call_method(
            &glue,
            "destroy",
            &Array::of1(&JsValue::from_str(container_id)),
        );
    }
}

/// Get the current map center as `(lng, lat)`.
pub fn get_center(container_id: &str) -> Option<(f64, f64)> {
    let glue = glue()?;
    let value = call_method(
        &glue,
        "getCenter",
        &Array::of1(&JsValue::from_str(container_id)),
    )
    .ok()?;
    let object = Object::from(value);
    let lng = Reflect::get(&object, &JsValue::from_str("lng"))
        .ok()?
        .as_f64()?;
    let lat = Reflect::get(&object, &JsValue::from_str("lat"))
        .ok()?
        .as_f64()?;
    Some((lng, lat))
}

/// Create or move the temporary draggable marker.
pub fn set_temp_marker(container_id: &str, lng: f64, lat: f64) {
    if let Some(glue) = glue() {
        let _ = call_method(
            &glue,
            "setTempMarker",
            &Array::of3(
                &JsValue::from_str(container_id),
                &JsValue::from_f64(lng),
                &JsValue::from_f64(lat),
            ),
        );
    }
}

/// Remove the temporary draggable marker.
pub fn remove_temp_marker(container_id: &str) {
    if let Some(glue) = glue() {
        let _ = call_method(
            &glue,
            "removeTempMarker",
            &Array::of1(&JsValue::from_str(container_id)),
        );
    }
}

/// Highlight (scale up) the active marker, or `None` to clear the highlight.
pub fn set_active_marker(container_id: &str, marker_id: Option<i64>) {
    if let Some(glue) = glue() {
        let value = match marker_id {
            Some(id) => JsValue::from_f64(id as f64),
            None => JsValue::null(),
        };
        let _ = call_method(
            &glue,
            "setActiveMarker",
            &Array::of2(&JsValue::from_str(container_id), &value),
        );
    }
}

/// Load the categorized emoji dataset from the glue bundle.
pub fn get_emoji_categories() -> Vec<super::models::EmojiCategory> {
    use super::models::{EmojiCategory, EmojiEntry};

    let Some(glue) = glue() else {
        return Vec::new();
    };
    let Ok(value) = call_method(&glue, "getEmojiData", &Array::new()) else {
        return Vec::new();
    };
    let object = Object::from(value);
    let keys = js_sys::Object::keys(&object);
    let mut categories = Vec::new();
    for key in keys.iter() {
        let Some(name) = key.as_string() else {
            continue;
        };
        let Ok(entries_value) = Reflect::get(&object, &key) else {
            continue;
        };
        let Ok(emojis) = serde_wasm_bindgen::from_value::<Vec<EmojiEntry>>(entries_value) else {
            continue;
        };
        categories.push(EmojiCategory { name, emojis });
    }
    categories
}
