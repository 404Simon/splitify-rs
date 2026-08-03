//! Group map page.
//!
//! A map-app style interface: the map fills the page, controls float over it,
//! and adding a marker is a guided flow (search for an address, drag a
//! temporary pin to the exact spot, fill in details, save). The marker list
//! slides in from the side.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
};

use crate::{
    components::{ErrorAlert, LoadingSpinner, MapCanvas, Navigation, PageHeader},
    features::{
        auth::{UserSession, use_logout},
        groups::handlers::get_group,
        maps::{
            CreateMapMarker, DeleteMapMarker, MapCommand, MapMarker, PlaceSearchResult,
            UpdateMapMarker, get_group_map_markers, get_map_config, search_places,
        },
    },
};

const MAP_ID: &str = "group-map";

/// Search for an address or place is triggered once the query reaches this
/// many characters.
const SEARCH_MIN_LENGTH: usize = 3;

fn format_coordinate(value: f64) -> String {
    format!("{value:.5}")
}

/// Group map page component.
#[must_use]
#[component]
pub fn GroupMap() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let navigate = use_navigate();
    let on_logout = use_logout();
    let params = use_params_map();

    // Effect to redirect if not authenticated
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate_clone("/login", Default::default());
        }
    });

    let group_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    });

    let group_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group(id).await }
    });

    let config_resource = LocalResource::new(|| async move { get_map_config().await });

    let markers_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_map_markers(id).await }
    });

    // Reactive marker list, fed from the resource so mutations can refetch it.
    let markers = RwSignal::new(Vec::<MapMarker>::new());
    Effect::new(move |_| {
        if let Some(Ok(list)) = markers_resource.get() {
            markers.set(list);
        }
    });

    // UI state
    let add_mode = RwSignal::new(false);
    let editing_id = RwSignal::new(None::<i64>);
    let selected_id = RwSignal::new(None::<i64>);
    let temp_marker = RwSignal::new(None::<(f64, f64)>);
    let list_open = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let commands = RwSignal::new(None::<MapCommand>);

    // Add/edit form state
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (address, set_address) = signal(String::new());

    // Address/place search state
    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal(Vec::<PlaceSearchResult>::new());
    let (searching, set_searching) = signal(false);
    let (search_failed, set_search_failed) = signal(false);
    let search_timer = RwSignal::new(None::<leptos::prelude::TimeoutHandle>);
    // Set when a search result is picked so the programmatic query change does
    // not trigger another round-trip to the geocoder.
    let search_lock = RwSignal::new(false);
    // Monotonic counter used to discard stale in-flight search responses.
    let search_seq = RwSignal::new(0u64);

    let create_action = ServerAction::<CreateMapMarker>::new();
    let update_action = ServerAction::<UpdateMapMarker>::new();
    let delete_action = ServerAction::<DeleteMapMarker>::new();

    let selected_marker = move || {
        selected_id
            .get()
            .and_then(|id| markers.get().into_iter().find(|marker| marker.id == id))
    };

    let cancel_adding = Callback::new(move |_: ()| {
        add_mode.set(false);
        editing_id.set(None);
        temp_marker.set(None);
        set_search_query.set(String::new());
        set_search_results.set(Vec::new());
        set_searching.set(false);
        search_lock.set(false);
        search_seq.set(search_seq.get_untracked() + 1);
        error_message.set(None);
    });

    let start_adding = Callback::new(move |_: ()| {
        add_mode.set(true);
        list_open.set(false);
        editing_id.set(None);
        selected_id.set(None);
        error_message.set(None);
        set_name.set(String::new());
        set_description.set(String::new());
        set_address.set(String::new());
        set_search_query.set(String::new());
        set_search_results.set(Vec::new());
        set_searching.set(false);
        search_lock.set(false);
        search_seq.set(search_seq.get_untracked() + 1);

        let (default_lng, default_lat) = config_resource
            .get_untracked()
            .and_then(|result| result.ok())
            .map(|config| (config.default_lng, config.default_lat))
            .unwrap_or((0.0, 0.0));

        // Place the temporary marker where the user is currently looking.
        #[cfg(feature = "hydrate")]
        let start = crate::features::maps::maplibre::get_center(MAP_ID)
            .unwrap_or((default_lng, default_lat));
        #[cfg(not(feature = "hydrate"))]
        let start = (default_lng, default_lat);

        temp_marker.set(Some(start));
    });

    let select_marker = Callback::new(move |id: Option<i64>| {
        add_mode.set(false);
        temp_marker.set(None);
        set_search_results.set(Vec::new());
        match id {
            Some(id) => {
                if let Some(marker) = markers.get_untracked().into_iter().find(|m| m.id == id) {
                    selected_id.set(Some(marker.id));
                    editing_id.set(None);
                    commands.set(Some(MapCommand::FlyTo {
                        lng: marker.longitude,
                        lat: marker.latitude,
                    }));
                }
            }
            None => {
                editing_id.set(None);
                selected_id.set(None);
            }
        }
    });

    let on_map_click = Callback::new(move |(lng, lat): (f64, f64)| {
        if add_mode.get_untracked() {
            temp_marker.set(Some((lng, lat)));
            set_search_results.set(Vec::new());
        } else {
            editing_id.set(None);
            selected_id.set(None);
        }
    });

    let on_temp_marker_moved = Callback::new(move |(lng, lat): (f64, f64)| {
        temp_marker.set(Some((lng, lat)));
    });

    let start_editing = Callback::new(move |marker: MapMarker| {
        add_mode.set(true);
        list_open.set(false);
        editing_id.set(Some(marker.id));
        selected_id.set(None);
        set_name.set(marker.name.clone());
        set_description.set(marker.description.clone().unwrap_or_default());
        set_address.set(marker.address.clone().unwrap_or_default());
        set_search_query.set(String::new());
        set_search_results.set(Vec::new());
        error_message.set(None);
        temp_marker.set(Some((marker.longitude, marker.latitude)));
    });

    let pick_search_result = Callback::new(move |result: PlaceSearchResult| {
        set_address.set(result.display_name.clone());
        temp_marker.set(Some((result.lon, result.lat)));
        // Invalidate any in-flight search so its late response cannot repopulate
        // the dropdown, then lock so the query change below is not re-searched.
        search_seq.set(search_seq.get_untracked() + 1);
        search_lock.set(true);
        set_search_query.set(result.display_name.clone());
        set_search_results.set(Vec::new());
        set_searching.set(false);
        commands.set(Some(MapCommand::FlyTo {
            lng: result.lon,
            lat: result.lat,
        }));
    });

    // Shared actions for the floating overlay controls.
    let on_toggle_add = Callback::new(move |_: ()| {
        if add_mode.get_untracked() {
            cancel_adding.run(());
        } else {
            start_adding.run(());
        }
    });
    let on_fit = Callback::new(move |_: ()| commands.set(Some(MapCommand::Fit)));
    let on_toggle_list = Callback::new(move |_: ()| list_open.set(!list_open.get()));
    let on_search_input = Callback::new(move |value: String| {
        search_lock.set(false);
        set_search_query.set(value);
        set_search_results.set(Vec::new());
    });

    // When the mobile carousel is swiped to another marker, fly the map there.
    let on_carousel_scroll = move |ev: leptos::ev::Event| {
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
            if let Some(id) = best_id
                && selected_id.get_untracked() != Some(id)
                && let Some(marker) = markers.get_untracked().into_iter().find(|m| m.id == id)
            {
                selected_id.set(Some(id));
                commands.set(Some(MapCommand::FlyTo {
                    lng: marker.longitude,
                    lat: marker.latitude,
                }));
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = ev;
    };

    // Debounced address/place search.
    Effect::new(move |_| {
        // Cancel any pending search from a previous keystroke.
        if let Some(handle) = search_timer.get_untracked() {
            handle.clear();
        }
        // A picked result changed the query programmatically — don't re-search.
        if search_lock.get_untracked() {
            search_lock.set(false);
            return;
        }
        let query = search_query.get().trim().to_string();
        if query.len() < SEARCH_MIN_LENGTH {
            set_search_results.set(Vec::new());
            set_searching.set(false);
            return;
        }
        let seq = search_seq.get_untracked() + 1;
        search_seq.set(seq);
        set_searching.set(true);
        set_search_failed.set(false);
        let handle = set_timeout_with_handle(
            move || {
                spawn_local(async move {
                    match search_places(query).await {
                        Ok(results) => {
                            if search_seq.get_untracked() == seq {
                                set_search_results.set(results);
                            }
                        }
                        Err(_) => {
                            if search_seq.get_untracked() == seq {
                                set_search_failed.set(true);
                            }
                        }
                    }
                    if search_seq.get_untracked() == seq {
                        set_searching.set(false);
                    }
                });
            },
            std::time::Duration::from_millis(400),
        )
        .ok();
        search_timer.set(handle);
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error_message.set(None);

        let marker_name = name.get().trim().to_string();
        if marker_name.is_empty() {
            error_message.set(Some("Marker name is required".to_string()));
            return;
        }

        let description = {
            let value = description.get().trim().to_string();
            if value.is_empty() { None } else { Some(value) }
        };
        let address = {
            let value = address.get().trim().to_string();
            if value.is_empty() { None } else { Some(value) }
        };

        let Some((lng, lat)) = temp_marker.get_untracked() else {
            error_message.set(Some("Place the marker on the map first".to_string()));
            return;
        };
        if !lat.is_finite()
            || !(-90.0..=90.0).contains(&lat)
            || !lng.is_finite()
            || !(-180.0..=180.0).contains(&lng)
        {
            error_message.set(Some("The marker position is invalid".to_string()));
            return;
        }

        if let Some(marker_id) = editing_id.get_untracked() {
            update_action.dispatch(UpdateMapMarker {
                marker_id,
                name: marker_name,
                description,
                address,
                latitude: lat,
                longitude: lng,
            });
        } else {
            create_action.dispatch(CreateMapMarker {
                group_id: group_id.get(),
                name: marker_name,
                description,
                address,
                latitude: lat,
                longitude: lng,
            });
        }
    };

    // Refetch markers and reset the form after any mutation succeeds.
    Effect::new(move |_| {
        if let Some(Ok(_)) = create_action.value().get() {
            cancel_adding.run(());
            markers_resource.refetch();
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(_)) = update_action.value().get() {
            cancel_adding.run(());
            markers_resource.refetch();
        }
    });
    Effect::new(move |_| {
        if let Some(result) = delete_action.value().get() {
            match result {
                Ok(_) => {
                    cancel_adding.run(());
                    selected_id.set(None);
                    markers_resource.refetch();
                }
                Err(e) => error_message.set(Some(e.to_string())),
            }
        }
    });

    // Surface server-side validation/authentication errors.
    Effect::new(move |_| {
        if let Some(Err(e)) = create_action.value().get() {
            error_message.set(Some(e.to_string()));
        }
    });
    Effect::new(move |_| {
        if let Some(Err(e)) = update_action.value().get() {
            error_message.set(Some(e.to_string()));
        }
    });

    let is_loading =
        Signal::derive(move || create_action.pending().get() || update_action.pending().get());

    let gid = group_id.get_untracked();

    view! {
        <Suspense fallback=LoadingSpinner>
            {move || {
                match user_resource.get() {
                    Some(Ok(Some(user))) => view! {
                        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
                            // On mobile the map takes the whole screen, so the
                            // navigation header is hidden there.
                            <div class="hidden md:block">
                                <Navigation username=user.username.clone() on_logout=on_logout />
                            </div>
                            <div class="md:py-6">
                                <div class="md:max-w-7xl md:mx-auto md:px-6 lg:px-8">
                                    <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                        {move || {
                                            match (group_resource.get(), config_resource.get()) {
                                                (Some(Ok(group)), Some(Ok(config))) => {
                                                    let is_admin = group.created_by == user.id;
                                                    view! {
                                                        <div>
                                                            <div class="hidden md:block">
                                                                <A href=format!("/groups/{gid}") attr:class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-900 dark:hover:text-indigo-300 text-sm inline-flex items-center mb-3">
                                                                    <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                                                                    </svg>
                                                                    "Back to Group"
                                                                </A>

                                                                <PageHeader
                                                                    title=format!("{} — Group Map", group.name)
                                                                    subtitle="Find and save the places your group wants to visit".to_string()
                                                                />
                                                            </div>

                                                            <div class="relative h-dvh md:h-[calc(100vh-17rem)] md:min-h-[440px] md:rounded-xl md:overflow-hidden md:border md:border-gray-200 md:dark:border-gray-700 md:shadow-sm bg-white dark:bg-gray-900">
                                                                <MapCanvas
                                                                    container_id=MAP_ID.to_string()
                                                                    config=config
                                                                    markers=markers
                                                                    add_mode=add_mode
                                                                    temp_marker=temp_marker
                                                                    commands=commands
                                                                    on_map_click=on_map_click
                                                                    on_marker_selected=select_marker
                                                                    on_temp_marker_moved=on_temp_marker_moved
                                                                />

                                                                // Corner controls: back (mobile), fit, list, add.
                                                                // During add mode they hide so the search + form take over.
                                                                {move || (!add_mode.get()).then(|| view! {
                                                                    <div class="absolute top-3 left-3 z-10 md:hidden">
                                                                        <MapBackButton href=format!("/groups/{gid}") />
                                                                    </div>
                                                                })}
                                                                {move || (!add_mode.get()).then(|| view! {
                                                                    <div class="absolute top-3 right-3 z-10">
                                                                        <MapFitButton on_click=on_fit />
                                                                    </div>
                                                                })}
                                                                {move || (!add_mode.get()).then(|| view! {
                                                                    <div class="absolute bottom-3 left-3 z-10">
                                                                        <MapListButton count=Signal::derive(move || markers.get().len()) open=list_open on_click=on_toggle_list />
                                                                    </div>
                                                                })}
                                                                {move || (!add_mode.get()).then(|| view! {
                                                                    <div class="absolute bottom-3 right-3 z-10">
                                                                        <MapAddButton add_mode=add_mode on_toggle=on_toggle_add />
                                                                    </div>
                                                                })}

                                                                // Address/place search overlay
                                                                {move || add_mode.get().then(|| view! {
                                                                    <div class="absolute top-3 left-3 right-3 z-20 md:left-1/2 md:right-auto md:w-[min(60%,26rem)] md:-translate-x-1/2">
                                                                        <SearchOverlay
                                                                            query=search_query
                                                                            on_input=on_search_input
                                                                            results=search_results
                                                                            searching=searching
                                                                            search_failed=search_failed
                                                                            on_pick=pick_search_result
                                                                        />
                                                                    </div>
                                                                })}

                                                                // Add/edit details form
                                                                {move || add_mode.get().then(|| view! {
                                                                    <div class="absolute inset-x-0 bottom-0 z-20 md:inset-x-auto md:left-auto md:right-3 md:bottom-3 md:w-96">
                                                                        <form
                                                                            on:submit=on_submit
                                                                            class="bg-white dark:bg-gray-800 rounded-t-2xl shadow-2xl border-t border-gray-200 dark:border-gray-700 p-4 space-y-3 md:rounded-xl md:border md:border-gray-200 md:dark:border-gray-700"
                                                                        >
                                                                            <div class="flex items-center justify-between">
                                                                                <h3 class="text-sm font-semibold text-gray-900 dark:text-white">
                                                                                    {move || if editing_id.get().is_some() { "Edit Location" } else { "Add a Location" }}
                                                                                </h3>
                                                                                <span class="text-xs text-gray-500 dark:text-gray-400 font-mono">
                                                                                    {move || {
                                                                                        temp_marker.get().map(|(lng, lat)| format!("{}, {}", format_coordinate(lat), format_coordinate(lng))).unwrap_or_default()
                                                                                    }}
                                                                                </span>
                                                                            </div>

                                                                            <ErrorAlert message=error_message />

                                                                            <input
                                                                                type="text"
                                                                                placeholder="Name *"
                                                                                required
                                                                                maxlength="255"
                                                                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white text-sm"
                                                                                prop:value=name
                                                                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                                                            />

                                                                            <input
                                                                                type="text"
                                                                                placeholder="Address (optional)"
                                                                                maxlength="500"
                                                                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white text-sm"
                                                                                prop:value=address
                                                                                on:input=move |ev| set_address.set(event_target_value(&ev))
                                                                            />

                                                                            <textarea
                                                                                rows="2"
                                                                                placeholder="Description (optional)"
                                                                                maxlength="500"
                                                                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white text-sm resize-none"
                                                                                prop:value=description
                                                                                on:input=move |ev| set_description.set(event_target_value(&ev))
                                                                            ></textarea>

                                                                            <div class="flex gap-2">
                                                                                <button
                                                                                    type="submit"
                                                                                    disabled=move || is_loading.get()
                                                                                    class="flex-1 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-400 text-white rounded-lg font-medium text-sm transition-colors"
                                                                                >
                                                                                    {move || {
                                                                                        if is_loading.get() {
                                                                                            "Saving..."
                                                                                        } else if editing_id.get().is_some() {
                                                                                            "Save Changes"
                                                                                        } else {
                                                                                            "Save Marker"
                                                                                        }
                                                                                    }}
                                                                                </button>
                                                                                <button
                                                                                    type="button"
                                                                                    on:click=move |_| cancel_adding.run(())
                                                                                    class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium text-sm transition-colors"
                                                                                >
                                                                                    "Cancel"
                                                                                </button>
                                                                            </div>
                                                                        </form>
                                                                    </div>
                                                                })}

                                                                // Marker list panel (desktop)
                                                                {move || list_open.get().then(|| view! {
                                                                    <div class="absolute inset-y-0 right-0 z-30 hidden md:flex w-96 bg-white dark:bg-gray-800 shadow-2xl border-l border-gray-200 dark:border-gray-700 flex-col">
                                                                        <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                                                                            <h3 class="text-sm font-semibold text-gray-900 dark:text-white">
                                                                                {move || format!("Locations ({})", markers.get().len())}
                                                                            </h3>
                                                                            <button
                                                                                on:click=move |_| list_open.set(false)
                                                                                class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                                                                                title="Close list"
                                                                            >
                                                                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                                                                </svg>
                                                                            </button>
                                                                        </div>

                                                                        {move || selected_marker().map(|marker| {
                                                                            let can_manage = marker.created_by == user.id || is_admin;
                                                                            view! {
                                                                                <div class="px-4 pt-3 pb-1 border-b border-gray-200 dark:border-gray-700">
                                                                                    <MarkerDetailsCard
                                                                                        marker=marker
                                                                                        can_manage=can_manage
                                                                                        on_edit=start_editing
                                                                                        on_delete=Callback::new(move |id: i64| { delete_action.dispatch(DeleteMapMarker { marker_id: id }); })
                                                                                    />
                                                                                </div>
                                                                            }
                                                                        })}

                                                                        <div class="flex-1 overflow-y-auto px-4 py-3">
                                                                            {move || {
                                                                                let list = markers.get();
                                                                                if list.is_empty() {
                                                                                    view! {
                                                                                        <p class="text-sm text-gray-500 dark:text-gray-400 italic">
                                                                                            "No locations yet — click \"Add Marker\" to save your first place."
                                                                                        </p>
                                                                                    }.into_any()
                                                                                } else {
                                                                                    view! {
                                                                                        <ul class="space-y-1">
                                                                                            {list.into_iter().map(|marker| {
                                                                                                let marker_id = marker.id;
                                                                                                let is_selected = selected_id.get() == Some(marker_id);
                                                                                                view! {
                                                                                                    <li>
                                                                                                        <button
                                                                                                            on:click=move |_| select_marker.run(Some(marker_id))
                                                                                                            class=move || if is_selected {
                                                                                                                "w-full text-left px-3 py-2.5 rounded-lg transition-colors bg-indigo-50 dark:bg-indigo-900/30"
                                                                                                            } else {
                                                                                                                "w-full text-left px-3 py-2.5 rounded-lg transition-colors hover:bg-gray-50 dark:hover:bg-gray-700"
                                                                                                            }
                                                                                                        >
                                                                                                            <p class="text-sm font-medium text-gray-900 dark:text-white truncate">{marker.name.clone()}</p>
                                                                                                            <p class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                                                                                                {marker.address.clone().unwrap_or_else(|| "by ".to_string() + &marker.creator_username)}
                                                                                                            </p>
                                                                                                        </button>
                                                                                                    </li>
                                                                                                }
                                                                                            }).collect_view()}
                                                                                        </ul>
                                                                                    }.into_any()
                                                                                }
                                                                            }}
                                                                        </div>
                                                                    </div>
                                                                })}

                                                                // Marker carousel (mobile) — floating detail cards, no sheet chrome.
                                                                // The List button (bottom-left) toggles it and reads "Close".
                                                                {move || list_open.get().then(|| view! {
                                                                    <div class="absolute inset-x-0 bottom-20 z-30 md:hidden">
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
                                                                                    <div class="flex items-stretch gap-3 overflow-x-auto snap-x snap-mandatory no-scrollbar px-4" on:scroll=on_carousel_scroll>
                                                                                        {list.into_iter().map(|marker| {
                                                                                            let marker_id = marker.id;
                                                                                            let can_manage = marker.created_by == user.id || is_admin;
                                                                                            let is_selected = selected_id.get() == Some(marker_id);
                                                                                            view! {
                                                                                                <div data-marker-id={marker_id} class="snap-center shrink-0 w-[80%] max-w-[20rem]">
                                                                                                    <MarkerCarouselCard
                                                                                                        marker=marker
                                                                                                        can_manage=can_manage
                                                                                                        is_selected=is_selected
                                                                                                        on_focus=Callback::new(move |id: i64| select_marker.run(Some(id)))
                                                                                                        on_edit=start_editing
                                                                                                        on_delete=Callback::new(move |id: i64| { delete_action.dispatch(DeleteMapMarker { marker_id: id }); })
                                                                                                    />
                                                                                                </div>
                                                                                            }
                                                                                        }).collect_view()}
                                                                                    </div>
                                                                                }.into_any()
                                                                            }
                                                                        }}
                                                                    </div>
                                                                })}
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                },
                                                (Some(Err(e)), _) | (_, Some(Err(e))) => view! {
                                                    <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                        <p class="text-sm text-red-700 dark:text-red-300">"Error: " {e.to_string()}</p>
                                                    </div>
                                                }.into_any(),
                                                _ => view! { <div>"Loading..."</div> }.into_any()
                                            }
                                        }}
                                    </Suspense>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    _ => LoadingSpinner().into_any()
                }
            }}
        </Suspense>
    }
}

/// Compact card showing a marker's details with edit/delete actions.
#[component]
fn MarkerDetailsCard(
    marker: MapMarker,
    can_manage: bool,
    #[prop(into)] on_edit: Callback<MapMarker>,
    #[prop(into)] on_delete: Callback<i64>,
) -> impl IntoView {
    let (confirming, set_confirming) = signal(false);
    let marker_id = marker.id;
    let edit_marker = marker.clone();

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-4">
            <h3 class="text-base font-semibold text-gray-900 dark:text-white truncate">{marker.name.clone()}</h3>
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
            {can_manage.then(|| view! {
                <div class="mt-3 flex gap-2">
                    <button
                        on:click=move |_| on_edit.run(edit_marker.clone())
                        class="flex-1 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium text-sm transition-colors"
                    >
                        "Edit"
                    </button>
                    <button
                        on:click=move |_| {
                            if confirming.get() {
                                on_delete.run(marker_id);
                            } else {
                                set_confirming.set(true);
                            }
                        }
                        class="flex-1 px-3 py-1.5 rounded-lg font-medium text-sm transition-colors border"
                        class:bg-red-600=move || confirming.get()
                        class:text-white=move || confirming.get()
                        class:border-red-600=move || confirming.get()
                        class:border-red-200=move || !confirming.get()
                        class:text-red-600=move || !confirming.get()
                        class:hover:bg-red-50=move || !confirming.get()
                    >
                        {move || if confirming.get() { "Confirm delete" } else { "Delete" }}
                    </button>
                </div>
            })}
        </div>
    }
}

/// Floating "back to group" button (used on mobile where the header is hidden).
#[component]
fn MapBackButton(href: String) -> impl IntoView {
    view! {
        <a
            href=href
            title="Back to group"
            class="inline-flex items-center justify-center w-10 h-10 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-full shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
            </svg>
        </a>
    }
}

/// Floating "Add"/"Cancel" action button (bottom-right).
#[component]
fn MapAddButton(add_mode: RwSignal<bool>, #[prop(into)] on_toggle: Callback<()>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_toggle.run(())
            class=move || format!(
                "inline-flex items-center gap-2 px-5 py-3 rounded-full font-semibold text-sm shadow-lg transition-colors {}",
                if add_mode.get() {
                    "bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700"
                } else {
                    "bg-indigo-600 hover:bg-indigo-700 text-white"
                }
            )
        >
            {move || if add_mode.get() {
                view! {
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                    "Cancel"
                }.into_any()
            } else {
                view! {
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    "Add"
                }.into_any()
            }}
        </button>
    }
}

/// Floating "fit to markers" button.
#[component]
fn MapFitButton(#[prop(into)] on_click: Callback<()>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click.run(())
            class="inline-flex items-center px-4 py-2 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg font-medium transition-colors shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700"
        >
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"/>
            </svg>
            "Fit"
        </button>
    }
}

/// Floating marker list toggle. Shows "Close" while the list is open.
#[component]
fn MapListButton(
    count: Signal<usize>,
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click.run(())
            class="inline-flex items-center px-4 py-2 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg font-medium transition-colors shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700"
        >
            {move || if open.get() {
                view! {
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                    "Close"
                }.into_any()
            } else {
                view! {
                    <>
                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7"/>
                        </svg>
                        "List"
                        <span class="ml-2 inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-semibold bg-indigo-100 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300">
                            {move || count.get()}
                        </span>
                    </>
                }.into_any()
            }}
        </button>
    }
}

/// Address/place search box with a results dropdown.
#[component]
fn SearchOverlay(
    #[prop(into)] query: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(into)] results: Signal<Vec<PlaceSearchResult>>,
    #[prop(into)] searching: Signal<bool>,
    #[prop(into)] search_failed: Signal<bool>,
    #[prop(into)] on_pick: Callback<PlaceSearchResult>,
) -> impl IntoView {
    view! {
        <div class="relative">
            <svg class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
            <input
                type="text"
                placeholder="Search for an address or place"
                autocomplete="off"
                class="w-full pl-10 pr-4 py-2.5 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                prop:value=query
                on:input=move |ev| on_input.run(event_target_value(&ev))
            />
            {move || {
                let spinner = searching.get().then(|| view! {
                    <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
                        <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-indigo-600"></div>
                    </div>
                });
                let failed = (!searching.get() && search_failed.get()).then(|| view! {
                    <p class="mt-1 px-3 py-2 text-xs text-red-600 dark:text-red-400">"Search is unavailable right now."</p>
                });
                view! { {spinner} {failed} }
            }}
        </div>

        {move || (!results.get().is_empty()).then(|| view! {
            <ul class="mt-1 rounded-lg overflow-hidden bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 shadow-lg max-h-64 overflow-y-auto">
                {results.get().into_iter().map(|result| {
                    let result_clone = result.clone();
                    view! {
                        <li>
                            <button
                                on:click=move |_| on_pick.run(result_clone.clone())
                                class="w-full text-left px-3 py-2.5 text-sm text-gray-800 dark:text-gray-200 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 flex items-start gap-2"
                            >
                                <svg class="w-4 h-4 mt-0.5 shrink-0 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
                                </svg>
                                <span class="line-clamp-2">{result.display_name}</span>
                            </button>
                        </li>
                    }
                }).collect_view()}
            </ul>
        })}
    }
}

/// Swipeable card used in the mobile marker carousel.
#[component]
fn MarkerCarouselCard(
    marker: MapMarker,
    can_manage: bool,
    is_selected: bool,
    #[prop(into)] on_focus: Callback<i64>,
    #[prop(into)] on_edit: Callback<MapMarker>,
    #[prop(into)] on_delete: Callback<i64>,
) -> impl IntoView {
    let (confirming, set_confirming) = signal(false);
    let marker_id = marker.id;
    let edit_marker = marker.clone();

    view! {
        <div class="h-full flex flex-col">
            <div class=move || format!(
                "bg-white dark:bg-gray-800 rounded-2xl border-2 p-4 shadow-lg transition-colors flex flex-col h-full {}",
                if is_selected {
                    "border-indigo-500 dark:border-indigo-400"
                } else {
                    "border-gray-200 dark:border-gray-700"
                }
            )>
                <div class="flex items-start justify-between gap-2">
                    <h3 class="text-base font-semibold text-gray-900 dark:text-white truncate">{marker.name.clone()}</h3>
                    <button
                        on:click=move |ev| {
                            ev.stop_propagation();
                            on_focus.run(marker_id);
                        }
                        class="shrink-0 inline-flex items-center gap-1 px-2 py-1 rounded-lg text-xs font-medium bg-indigo-100 dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300"
                    >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
                        </svg>
                        "Locate"
                    </button>
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
                {can_manage.then(|| view! {
                    <div class="flex gap-2 mt-3">
                        <button
                            on:click=move |ev| {
                                ev.stop_propagation();
                                on_edit.run(edit_marker.clone());
                            }
                            class="flex-1 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium text-sm transition-colors"
                        >
                            "Edit"
                        </button>
                        <button
                            on:click=move |ev| {
                                ev.stop_propagation();
                                if confirming.get() {
                                    on_delete.run(marker_id);
                                } else {
                                    set_confirming.set(true);
                                }
                            }
                            class="flex-1 px-3 py-1.5 rounded-lg font-medium text-sm transition-colors border"
                            class:bg-red-600=move || confirming.get()
                            class:text-white=move || confirming.get()
                            class:border-red-600=move || confirming.get()
                            class:border-red-200=move || !confirming.get()
                            class:text-red-600=move || !confirming.get()
                            class:hover:bg-red-50=move || !confirming.get()
                        >
                            {move || if confirming.get() { "Confirm" } else { "Delete" }}
                        </button>
                    </div>
                })}
            </div>
        </div>
    }
}
