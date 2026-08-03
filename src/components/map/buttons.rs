//! Floating overlay buttons used by the group map page.

use leptos::prelude::*;

/// Floating "back to group" button (used on mobile where the header is hidden).
#[component]
pub fn MapBackButton(href: String) -> impl IntoView {
    view! {
        <a
            href=href
            title="Back to group"
            class="inline-flex items-center justify-center w-10 h-10 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
            </svg>
        </a>
    }
}

/// Floating "Add"/"Cancel" action button (bottom-right).
#[component]
pub fn MapAddButton(
    add_mode: RwSignal<bool>,
    #[prop(into)] on_toggle: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_toggle.run(())
            class=move || format!(
                "inline-flex items-center justify-center gap-2 h-10 px-4 rounded-lg font-medium text-sm shadow-sm transition-colors {}",
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
pub fn MapFitButton(#[prop(into)] on_click: Callback<()>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click.run(())
            class="inline-flex items-center justify-center h-10 px-4 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg font-medium text-sm transition-colors shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700"
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
pub fn MapListButton(
    count: Signal<usize>,
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| on_click.run(())
            class="inline-flex items-center justify-center h-10 px-4 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 rounded-lg font-medium text-sm transition-colors shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700"
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
