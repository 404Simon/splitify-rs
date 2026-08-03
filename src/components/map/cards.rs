//! Marker detail cards and the mobile marker carousel.

use leptos::prelude::*;

use crate::features::maps::models::MapMarker;
use crate::features::maps::utils::{can_manage_marker, format_coordinate, google_maps_nav_url};

/// Compact card showing a marker's details with edit/navigate actions.
#[component]
pub fn MarkerDetailsCard(
    marker: MapMarker,
    can_manage: bool,
    #[prop(into)] on_edit: Callback<MapMarker>,
) -> impl IntoView {
    let edit_marker = marker.clone();

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-4">
            <div class="flex items-center gap-2 min-w-0">
                <span class="text-xl shrink-0">{marker.emoji.clone()}</span>
                <h3 class="text-base font-semibold text-gray-900 dark:text-white truncate">{marker.name.clone()}</h3>
            </div>
            {marker.address.clone().map(|addr| view! {
                <p class="text-sm text-gray-600 dark:text-gray-400 mt-0.5 truncate">{addr}</p>
            })}
            {marker.description.clone().map(|desc| view! {
                <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">{desc}</p>
            })}
            <dl class="mt-3 space-y-1 text-xs text-gray-500 dark:text-gray-400">
                <div class="flex justify-between">
                    <dt>"Added by"</dt>
                    <dd class="font-medium">{marker.creator_username.clone()}</dd>
                </div>
                <div class="flex justify-between font-mono">
                    <dt>"Coordinates"</dt>
                    <dd>{format!("{}, {}", format_coordinate(marker.latitude), format_coordinate(marker.longitude))}</dd>
                </div>
            </dl>
            <div class="mt-3 flex gap-2">
                {can_manage.then(|| view! {
                    <button
                        on:click=move |_| on_edit.run(edit_marker.clone())
                        class="flex-1 px-3 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium text-sm transition-colors"
                    >
                        "Edit"
                    </button>
                })}
                <a
                    href=google_maps_nav_url(&marker)
                    target="_blank"
                    rel="noopener"
                    class="flex flex-1 items-center justify-center gap-2 px-3 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-medium text-sm transition-colors"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
                    </svg>
                    "Navigate"
                </a>
            </div>
        </div>
    }
}

/// Swipeable card used in the mobile marker carousel.
#[component]
pub fn MarkerCarouselCard(
    marker: MapMarker,
    can_manage: bool,
    is_selected: bool,
    #[prop(into)] on_focus: Callback<i64>,
    #[prop(into)] on_edit: Callback<MapMarker>,
) -> impl IntoView {
    let marker_id = marker.id;
    let edit_marker = marker.clone();

    view! {
        <div class="h-full flex flex-col">
            <div
                on:click=move |_| on_focus.run(marker_id)
                class=move || format!(
                    "bg-white dark:bg-gray-800 rounded-2xl border-2 p-4 shadow-lg transition-colors flex flex-col h-full cursor-pointer {}",
                    if is_selected {
                        "border-indigo-500 dark:border-indigo-400"
                    } else {
                        "border-gray-200 dark:border-gray-700"
                    }
                )
            >
                <div class="flex items-center gap-2 min-w-0">
                    <span class="text-xl shrink-0">{marker.emoji.clone()}</span>
                    <h3 class="text-base font-semibold text-gray-900 dark:text-white truncate">{marker.name.clone()}</h3>
                </div>
                {marker.address.clone().map(|addr| view! {
                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1 truncate">{addr}</p>
                })}
                {marker.description.clone().map(|desc| view! {
                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1 line-clamp-2">{desc}</p>
                })}
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-2 font-mono">
                    {format!("{}, {}", format_coordinate(marker.latitude), format_coordinate(marker.longitude))}
                </p>
                <div class="flex gap-2 mt-auto pt-3">
                    {can_manage.then(|| view! {
                        <button
                            on:click=move |ev| {
                                ev.stop_propagation();
                                on_edit.run(edit_marker.clone());
                            }
                            class="flex-1 px-3 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium text-sm transition-colors"
                        >
                            "Edit"
                        </button>
                    })}
                    <a
                        href=google_maps_nav_url(&marker)
                        target="_blank"
                        rel="noopener"
                        on:click=move |ev| ev.stop_propagation()
                        class="flex flex-1 items-center justify-center gap-2 px-3 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-medium text-sm transition-colors"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
                        </svg>
                        "Navigate"
                    </a>
                </div>
            </div>
        </div>
    }
}

/// Swipeable carousel of marker cards shown on mobile when the list is open.
///
/// Swiping selects the nearest card and, once the swipe settles (debounced),
/// pans the map camera to it through `camera_target`.
#[component]
pub fn MarkerCarousel(
    markers: RwSignal<Vec<MapMarker>>,
    user_id: i64,
    is_admin: bool,
    selected_id: RwSignal<Option<i64>>,
    camera_target: RwSignal<Option<i64>>,
    camera_timer: RwSignal<Option<TimeoutHandle>>,
    #[prop(into)] on_focus: Callback<i64>,
    #[prop(into)] on_edit: Callback<MapMarker>,
) -> impl IntoView {
    // `camera_target`/`camera_timer` only drive the camera from the client-side
    // scroll handler, so they are unused in the server-rendered shell.
    #[cfg(not(feature = "hydrate"))]
    let _ = (camera_target, camera_timer);

    // When the carousel is swiped to another marker, gently pan the map there.
    // The highlight updates immediately, but the camera only moves once the
    // swipe settles (debounced) so a fast swipe doesn't restart the pan on
    // every scroll event.
    let on_scroll = move |ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;

            let target = event_target::<web_sys::HtmlElement>(&ev);
            let center = target.scroll_left() + target.client_width() / 2;
            let mut best_id = None;
            let mut best_dist = i32::MAX;
            let children = target.children();
            for i in 0..children.length() {
                if let Some(child) = children.item(i)
                    && let Some(element) = child.dyn_into::<web_sys::HtmlElement>().ok()
                {
                    let mid = element.offset_left() + element.offset_width() / 2;
                    let dist = (mid - center).abs();
                    if dist < best_dist
                        && let Some(id) = element
                            .dataset()
                            .get("markerId")
                            .and_then(|value| value.parse::<i64>().ok())
                    {
                        best_id = Some(id);
                        best_dist = dist;
                    }
                }
            }
            if let Some(id) = best_id {
                if selected_id.get_untracked() != Some(id) {
                    selected_id.set(Some(id));
                }
                if camera_target.get_untracked() != Some(id) {
                    if let Some(handle) = camera_timer.get_untracked() {
                        handle.clear();
                    }
                    camera_timer.set(
                        set_timeout_with_handle(
                            move || camera_target.set(Some(id)),
                            std::time::Duration::from_millis(180),
                        )
                        .ok(),
                    );
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = ev;
    };

    view! {
        {move || {
            let list = markers.get();
            if list.is_empty() {
                view! {
                    <p class="px-4 text-sm text-gray-500 dark:text-gray-400 italic">
                        "No locations yet — tap \"Add\" to save your first place."
                    </p>
                }.into_any()
            } else {
                view! {
                    <div class="flex items-stretch gap-3 overflow-x-auto snap-x snap-mandatory no-scrollbar px-4" on:scroll=on_scroll>
                        {list.into_iter().map(|marker| {
                            let marker_id = marker.id;
                            let can_manage = can_manage_marker(&marker, user_id, is_admin);
                            let is_selected = selected_id.get() == Some(marker_id);
                            view! {
                                <div data-marker-id={marker_id} class="snap-center shrink-0 w-[80%] max-w-[20rem]">
                                    <MarkerCarouselCard
                                        marker=marker
                                        can_manage=can_manage
                                        is_selected=is_selected
                                        on_focus=on_focus
                                        on_edit=on_edit
                                    />
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }
        }}
    }
}
