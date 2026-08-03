//! Address/place search box with a results dropdown.

use leptos::prelude::*;

use crate::features::maps::models::PlaceSearchResult;

/// Address/place search box with a results dropdown.
#[component]
pub fn SearchOverlay(
    #[prop(into)] query: Signal<String>,
    #[prop(into)] on_input: Callback<String>,
    #[prop(into)] results: Signal<Vec<PlaceSearchResult>>,
    #[prop(into)] searching: Signal<bool>,
    #[prop(into)] search_failed: Signal<bool>,
    #[prop(into)] on_pick: Callback<PlaceSearchResult>,
) -> impl IntoView {
    let input_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let clear = move |_| {
        on_input.run(String::new());
        // The clear button stole focus; hand it back to the input so the user
        // can immediately type a new query.
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    view! {
        <div class="relative">
            <svg class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
            <input
                type="text"
                placeholder="Search for an address or place"
                autocomplete="off"
                node_ref=input_ref
                class=move || format!(
                    "w-full pl-10 py-2.5 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 {}",
                    if query.get().is_empty() && !searching.get() { "pr-4" } else { "pr-10" }
                )
                prop:value=query
                on:input=move |ev| on_input.run(event_target_value(&ev))
            />
            {move || {
                let failed = (!searching.get() && search_failed.get()).then(|| view! {
                    <p class="mt-1 px-3 py-2 text-xs text-red-600 dark:text-red-400">"Search is unavailable right now."</p>
                });
                let right = if searching.get() {
                    view! {
                        <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
                            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-indigo-600"></div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <button
                            type="button"
                            on:click=clear
                            title="Clear search"
                            aria-label="Clear search"
                            class=move || format!(
                                "absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 {}",
                                if query.get().is_empty() { "invisible" } else { "" }
                            )
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    }.into_any()
                };
                view! { {right} {failed} }
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
