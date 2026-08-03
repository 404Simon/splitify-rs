//! Interactive map canvas backed by MapLibre GL JS.
//!
//! The component renders a container `<div>` and, once the client hydrates,
//! drives the bundled glue module (`window.SplitifyMap`, see
//! `features::maps::maplibre`). All MapLibre specifics live in the JS glue;
//! this component only orchestrates the reactive state.

use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::features::maps::maplibre::MapCallbacks;
use crate::features::maps::models::{MapCommand, MapConfig, MapMarker};

/// Interactive map canvas for a group's map.
///
/// Props:
/// - `container_id`: stable DOM id used both as the container and as the map
///   runtime key on the client.
/// - `config`: server-provided map configuration (style URL and initial view).
/// - `markers`: the markers to render; kept in sync reactively.
/// - `add_mode`: when enabled, clicking the map reports coordinates instead of
///   selecting a marker.
/// - `temp_marker`: position of the temporary draggable marker (`None` hides
///   it). Moved by the user it fires `on_temp_marker_moved`.
/// - `selected_marker`: the currently selected marker id, shown scaled up.
/// - `commands`: imperative one-shot commands (`FlyTo`, `Fit`).
/// - `on_map_click`: fired with `(lng, lat)` whenever the map is clicked.
/// - `on_marker_selected`: fired with the marker id (or `None`) when a marker
///   is selected or the selection is cleared.
/// - `on_temp_marker_moved`: fired with `(lng, lat)` when the user drags the
///   temporary marker to a new position.
#[component]
#[allow(unused)]
pub fn MapCanvas(
    container_id: String,
    config: MapConfig,
    markers: RwSignal<Vec<MapMarker>>,
    add_mode: RwSignal<bool>,
    temp_marker: RwSignal<Option<(f64, f64)>>,
    selected_marker: RwSignal<Option<i64>>,
    commands: RwSignal<Option<MapCommand>>,
    #[prop(into)] on_map_click: Callback<(f64, f64)>,
    #[prop(into)] on_marker_selected: Callback<Option<i64>>,
    #[prop(into)] on_temp_marker_moved: Callback<(f64, f64)>,
) -> impl IntoView {
    let map_ready = RwSignal::new(false);

    #[cfg(feature = "hydrate")]
    {
        use crate::features::maps::maplibre;

        let cancelled = RwSignal::new(false);
        let stored = StoredValue::new_local(None::<MapCallbacks>);

        let init_container = container_id.clone();
        let init_config = config.clone();
        request_animation_frame(move || {
            init_map_with_retry(
                0,
                init_container,
                init_config,
                map_ready,
                cancelled,
                stored,
                on_map_click,
                on_marker_selected,
                on_temp_marker_moved,
            );
        });

        // Keep the rendered markers in sync with the reactive state. The first
        // time markers arrive (after the map is ready) we fit the viewport.
        let sync_container = container_id.clone();
        let had_markers = RwSignal::new(false);
        Effect::new(move |_| {
            if !map_ready.get() {
                return;
            }
            let list = markers.get();
            let _ = maplibre::set_markers(&sync_container, &list);
            if !list.is_empty() && !had_markers.get_untracked() {
                maplibre::fit_markers(&sync_container);
                had_markers.set(true);
            }
        });

        // Execute one-shot commands from the page.
        let command_container = container_id.clone();
        Effect::new(move |_| {
            if let Some(command) = commands.get() {
                match command {
                    MapCommand::Fit => maplibre::fit_markers(&command_container),
                    MapCommand::FlyTo { lng, lat } => {
                        maplibre::fly_to(&command_container, lng, lat);
                    }
                    MapCommand::CenterOn { lng, lat } => {
                        maplibre::center_on(&command_container, lng, lat);
                    }
                }
                commands.set(None);
            }
        });

        // Toggle the crosshair cursor for "pick a location" mode.
        let add_container = container_id.clone();
        Effect::new(move |_| {
            maplibre::set_add_mode(&add_container, add_mode.get());
        });

        // Keep the temporary draggable marker in sync.
        let temp_container = container_id.clone();
        Effect::new(move |_| {
            if let Some((lng, lat)) = temp_marker.get() {
                maplibre::set_temp_marker(&temp_container, lng, lat);
            } else {
                maplibre::remove_temp_marker(&temp_container);
            }
        });

        // Highlight the selected marker.
        let active_container = container_id.clone();
        Effect::new(move |_| {
            maplibre::set_active_marker(&active_container, selected_marker.get());
        });

        let cleanup_container = container_id.clone();
        on_cleanup(move || {
            cancelled.set(true);
            stored.dispose();
            maplibre::destroy_map(&cleanup_container);
        });
    }

    view! {
        <div id=container_id class="w-full h-full"></div>
    }
}

/// Initialise the map once the glue module is available, retrying a bounded
/// number of times to tolerate the module script loading after hydration.
#[cfg(feature = "hydrate")]
#[allow(clippy::too_many_arguments)]
fn init_map_with_retry(
    attempt: usize,
    container_id: String,
    config: MapConfig,
    map_ready: RwSignal<bool>,
    cancelled: RwSignal<bool>,
    stored: StoredValue<Option<MapCallbacks>, LocalStorage>,
    on_map_click: Callback<(f64, f64)>,
    on_marker_selected: Callback<Option<i64>>,
    on_temp_marker_moved: Callback<(f64, f64)>,
) {
    use crate::features::maps::maplibre;
    use std::time::Duration;
    use wasm_bindgen::JsValue;

    if cancelled.get_untracked() {
        return;
    }

    if !maplibre::glue_loaded() {
        if attempt < 100 {
            set_timeout(
                move || {
                    init_map_with_retry(
                        attempt + 1,
                        container_id,
                        config,
                        map_ready,
                        cancelled,
                        stored,
                        on_map_click,
                        on_marker_selected,
                        on_temp_marker_moved,
                    );
                },
                Duration::from_millis(100),
            );
        } else {
            maplibre::log_error("MapLibre glue did not load; the map is unavailable");
        }
        return;
    }

    if let Err(error) = maplibre::create_map(
        &container_id,
        &config.style_url,
        &config.dark_style_url,
        config.default_lng,
        config.default_lat,
        config.default_zoom,
    ) {
        maplibre::log_error(&format!("Failed to initialise map: {error}"));
        return;
    }

    let on_map_load = move || map_ready.set(true);

    let on_map_click = move |coords: JsValue| {
        if !map_ready.get_untracked() {
            return;
        }
        let array = js_sys::Array::from(&coords);
        let lng = array.get(0).as_f64().unwrap_or(0.0);
        let lat = array.get(1).as_f64().unwrap_or(0.0);
        on_map_click.run((lng, lat));
    };

    let on_marker_selected = move |id: JsValue| {
        if !map_ready.get_untracked() {
            return;
        }
        on_marker_selected.run(id.as_f64().map(|value| value as i64));
    };

    let on_temp_marker_moved = move |coords: JsValue| {
        if !map_ready.get_untracked() {
            return;
        }
        let array = js_sys::Array::from(&coords);
        let lng = array.get(0).as_f64().unwrap_or(0.0);
        let lat = array.get(1).as_f64().unwrap_or(0.0);
        on_temp_marker_moved.run((lng, lat));
    };

    let callbacks = MapCallbacks::new(
        on_map_load,
        on_map_click,
        on_marker_selected,
        on_temp_marker_moved,
    );
    if let Err(error) = maplibre::set_callbacks(&container_id, &callbacks) {
        maplibre::log_error(&format!("Failed to attach map callbacks: {error}"));
    }
    stored.set_value(Some(callbacks));
}
