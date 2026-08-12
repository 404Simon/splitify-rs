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
    components::{
        EmojiPicker, ErrorAlert, LoadingSpinner, MapAddButton, MapBackButton, MapCanvas,
        MapFitButton, MapListButton, MapLocateButton, MarkerCarousel, MarkerDetailsCard,
        Navigation, PageHeader, SearchOverlay,
    },
    features::{
        auth::{UserSession, use_logout},
        groups::handlers::get_group,
        maps::utils::{can_manage_marker, format_coordinate},
        maps::{
            CreateMapMarker, DEFAULT_MARKER_EMOJI, DeleteMapMarker, EmojiCategory, MapCommand,
            MapMarker, PlaceSearchResult, UpdateMapMarker, get_group_map_markers, get_map_config,
            search_places,
        },
    },
};

const MAP_ID: &str = "group-map";

/// Search for an address or place is triggered once the query reaches this
/// many characters.
const SEARCH_MIN_LENGTH: usize = 3;

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
    // True while the "locate me" button waits for the browser geolocation.
    let locating = RwSignal::new(false);

    // Camera target: the marker the map should center on. A single effect
    // translates it into a gentle `CenterOn` command so every deliberate
    // selection path (map click, list, card tap) behaves the same and no two
    // callers fight over the camera. Carousel swipes bypass this and fly
    // directly via `MapCommand::FlyTo`.
    let camera_target = RwSignal::new(None::<i64>);
    // Selection that originated outside the carousel (map click / list / card
    // click). The carousel auto-scrolls to it, but NOT to swipe-driven
    // selection, which would fight the user's finger.
    let external_selection = RwSignal::new(None::<i64>);

    // Add/edit form state
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (address, set_address) = signal(String::new());
    let (emoji, set_emoji) = signal(DEFAULT_MARKER_EMOJI.to_string());

    // Address/place search state
    let (search_query, set_search_query) = signal(String::new());
    let (search_results, set_search_results) = signal(Vec::<PlaceSearchResult>::new());
    let (searching, set_searching) = signal(false);
    let (search_failed, set_search_failed) = signal(false);
    let search_timer = RwSignal::new(None::<leptos::prelude::TimeoutHandle>);
    // Monotonic counter used to discard stale in-flight search responses.
    let search_seq = RwSignal::new(0u64);

    // Debounced search triggered only by user input (`on_search_input`).
    // Picking a result fills the query field programmatically without going
    // through this path, so no "suppress the next run" flag is needed.
    let run_search = Callback::new(move |raw: String| {
        let query = raw.trim().to_string();
        if let Some(handle) = search_timer.get_untracked() {
            handle.clear();
        }
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

    let create_action = ServerAction::<CreateMapMarker>::new();
    let update_action = ServerAction::<UpdateMapMarker>::new();
    let delete_action = ServerAction::<DeleteMapMarker>::new();

    // Emoji picker dataset, loaded from the glue bundle on the client.
    let emoji_categories = RwSignal::new(Vec::<EmojiCategory>::new());
    #[cfg(feature = "hydrate")]
    crate::features::maps::maplibre::when_glue_ready(move || {
        emoji_categories.set(crate::features::maps::maplibre::get_emoji_categories());
    });

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
        if let Some(handle) = search_timer.get_untracked() {
            handle.clear();
        }
        search_seq.set(search_seq.get_untracked() + 1);
        error_message.set(None);
    });

    let start_adding = Callback::new(move |_: ()| {
        add_mode.set(true);
        list_open.set(false);
        editing_id.set(None);
        selected_id.set(None);
        external_selection.set(None);
        camera_target.set(None);
        error_message.set(None);
        set_name.set(String::new());
        set_description.set(String::new());
        set_address.set(String::new());
        set_emoji.set(DEFAULT_MARKER_EMOJI.to_string());
        set_search_query.set(String::new());
        set_search_results.set(Vec::new());
        set_searching.set(false);
        if let Some(handle) = search_timer.get_untracked() {
            handle.clear();
        }
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
                    list_open.set(true);
                    external_selection.set(Some(marker.id));
                    camera_target.set(Some(marker.id));
                }
            }
            None => {
                editing_id.set(None);
                selected_id.set(None);
                external_selection.set(None);
                camera_target.set(None);
            }
        }
    });

    // Selecting a marker switches the camera gently instead of flying across
    // the map.
    let focus_marker = Callback::new(move |id: i64| {
        if let Some(marker) = markers.get_untracked().into_iter().find(|m| m.id == id) {
            selected_id.set(Some(marker.id));
            external_selection.set(Some(marker.id));
            camera_target.set(Some(marker.id));
        }
    });

    let on_map_click = Callback::new(move |(lng, lat): (f64, f64)| {
        if add_mode.get_untracked() {
            temp_marker.set(Some((lng, lat)));
            set_search_results.set(Vec::new());
        } else {
            editing_id.set(None);
            selected_id.set(None);
            external_selection.set(None);
            camera_target.set(None);
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
        set_emoji.set(marker.emoji.clone());
        set_search_query.set(String::new());
        set_search_results.set(Vec::new());
        error_message.set(None);
        temp_marker.set(Some((marker.longitude, marker.latitude)));
    });

    let pick_search_result = Callback::new(move |result: PlaceSearchResult| {
        set_address.set(result.display_name.clone());
        temp_marker.set(Some((result.lon, result.lat)));
        // Invalidate any in-flight search so its late response cannot
        // repopulate the dropdown, and cancel any pending debounce so the
        // programmatic query change below is not re-searched.
        search_seq.set(search_seq.get_untracked() + 1);
        if let Some(handle) = search_timer.get_untracked() {
            handle.clear();
        }
        set_search_query.set(result.display_name.clone());
        set_search_results.set(Vec::new());
        set_searching.set(false);
        commands.set(Some(MapCommand::FlyTo {
            lng: result.lon,
            lat: result.lat,
            zoom: None,
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
        set_search_query.set(value.clone());
        set_search_results.set(Vec::new());
        run_search.run(value);
    });

    // "Locate me": query the browser for the user's position, drop the blue
    // "you are here" dot on the map and fly the camera to it.
    let on_locate = Callback::new(move |_: ()| {
        if locating.get_untracked() {
            return;
        }
        locating.set(true);
        error_message.set(None);

        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;

            let Some(geolocation) = web_sys::window()
                .map(|window| window.navigator())
                .and_then(|navigator| navigator.geolocation().ok())
            else {
                locating.set(false);
                error_message.set(Some(
                    "Your browser does not support geolocation".to_string(),
                ));
                return;
            };

            // `once_into_js` deliberately leaks: the success/error callbacks
            // fire exactly once, so freeing the wasm closure on the way out is
            // fine for a one-shot request.
            let on_success = Closure::once_into_js(move |position: web_sys::Position| {
                let coords = position.coords();
                let lng = coords.longitude();
                let lat = coords.latitude();
                locating.set(false);
                crate::features::maps::maplibre::set_user_location(MAP_ID, lng, lat);
                commands.set(Some(MapCommand::FlyTo {
                    lng,
                    lat,
                    zoom: Some(16.0),
                }));
            });
            let on_error = Closure::once_into_js(move |error: web_sys::PositionError| {
                locating.set(false);
                error_message.set(Some(format!(
                    "Could not determine your location: {}",
                    error.message()
                )));
            });

            let _ = geolocation.get_current_position_with_error_callback(
                on_success.unchecked_ref(),
                Some(on_error.unchecked_ref()),
            );
        }
        #[cfg(not(feature = "hydrate"))]
        {
            locating.set(false);
        }
    });

    // One gentle pan per camera-target change, shared by every deliberate
    // selection path (map click, list, card tap).
    Effect::new(move |_| {
        if let Some(id) = camera_target.get()
            && let Some(marker) = markers.get_untracked().into_iter().find(|m| m.id == id)
        {
            commands.set(Some(MapCommand::CenterOn {
                lng: marker.longitude,
                lat: marker.latitude,
            }));
        }
    });

    // Carousel swipes fly the camera with the zoom animation (same as picking
    // a search result), instead of the gentle pan used for deliberate clicks.
    let carousel_camera = Callback::new(move |(lng, lat): (f64, f64)| {
        commands.set(Some(MapCommand::FlyTo {
            lng,
            lat,
            zoom: None,
        }));
    });

    // When the mobile carousel opens (or the selection was made outside it —
    // map click, list, card), snap it to the selected marker's card. Swipe
    // driven selection is deliberately excluded: scrolling the carousel to the
    // card the user is already looking at would fight their finger.
    Effect::new(move |_| {
        let open = list_open.get();
        let id = external_selection.get();
        #[cfg(feature = "hydrate")]
        {
            if open
                && let Some(id) = id
                && let Ok(Some(card)) =
                    document().query_selector(&format!(".snap-x [data-marker-id=\"{id}\"]"))
            {
                card.scroll_into_view();
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = (open, id);
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
                emoji: emoji.get_untracked(),
                latitude: lat,
                longitude: lng,
            });
        } else {
            create_action.dispatch(CreateMapMarker {
                group_id: group_id.get(),
                name: marker_name,
                description,
                address,
                emoji: emoji.get_untracked(),
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
                                                                    title=format!("{} Map", group.name)
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
                                                                        selected_marker=selected_id
                                                                        commands=commands
                                                                        on_map_click=on_map_click
                                                                        on_marker_selected=select_marker
                                                                        on_temp_marker_moved=on_temp_marker_moved
                                                                    />

                                                                    // Subtle tile attribution (replaces MapLibre's control with the logo).
                                                                    <a
                                                                        href="https://www.openfreemap.org/"
                                                                        target="_blank"
                                                                        rel="noopener"
                                                                        class="absolute bottom-1 left-1/2 -translate-x-1/2 z-10 text-[10px] leading-none text-gray-400/80 dark:text-gray-500/80 hover:text-indigo-500 dark:hover:text-indigo-400"
                                                                    >
                                                                        "© OpenMapTiles © OpenStreetMap"
                                                                    </a>


                                                                    // Corner controls: back (mobile), fit, list, add.
                                                                    // During add mode they hide so the search + form take over.
                                                                    {move || (!add_mode.get()).then(|| view! {
                                                                        <div class="absolute top-3 left-3 z-10 md:hidden">
                                                                            <MapBackButton href=format!("/groups/{gid}") />
                                                                        </div>
                                                                    })}
                                                                    // Locate/geolocation errors (only visible outside add mode;
                                                                    // the form shows them too via its own ErrorAlert).
                                                                    {move || (!add_mode.get() && error_message.get().is_some()).then(|| view! {
                                                                        <div class="absolute top-3 left-1/2 -translate-x-1/2 z-20 w-[min(90%,24rem)]">
                                                                            <ErrorAlert message=error_message />
                                                                        </div>
                                                                    })}
                                                                    {move || (!add_mode.get()).then(|| view! {
                                                                        <div class="absolute top-3 right-3 z-10 flex flex-col gap-2">
                                                                            <MapFitButton on_click=on_fit />
                                                                            <MapLocateButton locating=locating on_click=on_locate />
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

                                                                                <EmojiPicker
                                                                                    emoji=emoji
                                                                                    categories=emoji_categories
                                                                                    on_select=Callback::new(move |value: String| set_emoji.set(value))
                                                                                />

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
                                                                                {move || editing_id.get().map(|id| view! {
                                                                                    <button
                                                                                        type="button"
                                                                                        on:click=move |_| {
                                                                                            delete_action.dispatch(DeleteMapMarker { marker_id: id });
                                                                                        }
                                                                                        class="w-full px-4 py-2 rounded-lg font-medium text-sm transition-colors border border-red-200 dark:border-red-900 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30"
                                                                                    >
                                                                                        "Delete Location"
                                                                                    </button>
                                                                                })}
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
                                                                                let can_manage = can_manage_marker(&marker, user.id, is_admin);
                                                                                view! {
                                                                                    <div class="px-4 pt-3 pb-1 border-b border-gray-200 dark:border-gray-700">
                                                                                        <MarkerDetailsCard
                                                                                            marker=marker
                                                                                            can_manage=can_manage
                                                                                            on_edit=start_editing
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
                                                                                                                <div class="flex items-center gap-2 min-w-0">
                                                                                                                    <span class="shrink-0">{marker.emoji.clone()}</span>
                                                                                                                    <p class="text-sm font-medium text-gray-900 dark:text-white truncate">{marker.name.clone()}</p>
                                                                                                                </div>
                                                                                                                <p class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                                                                                                    {marker.address.clone().unwrap_or_else(|| format!("by {}", marker.creator_username))}
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
                                                                            <MarkerCarousel
                                                                                markers=markers
                                                                                user_id=user.id
                                                                                is_admin=is_admin
                                                                                selected_id=selected_id
                                                                                on_focus=focus_marker
                                                                                on_edit=start_editing
                                                                                on_camera=carousel_camera
                                                                            />
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
