//! Emoji picker with search and category filtering (data from `emoji.json`).
//! Renders as a simple field that opens a modal popup with the full picker.

use leptos::prelude::*;

use crate::features::maps::models::{DEFAULT_MARKER_EMOJI, EmojiCategory};

#[component]
pub fn EmojiPicker(
    #[prop(into)] emoji: Signal<String>,
    categories: RwSignal<Vec<EmojiCategory>>,
    #[prop(into)] on_select: Callback<String>,
) -> impl IntoView {
    let (open, set_open) = signal(false);
    let (query, set_query) = signal(String::new());
    let (active_category, set_active_category) = signal("All".to_string());

    let filtered = move || {
        let needle = query.get().trim().to_lowercase();
        let category = active_category.get();
        categories
            .get()
            .into_iter()
            .filter(|c| category == "All" || c.name == category)
            .flat_map(|c| c.emojis)
            .filter(|entry| needle.is_empty() || entry.title.to_lowercase().contains(&needle))
            .take(300)
            .collect::<Vec<_>>()
    };

    let open_picker = move || {
        set_query.set(String::new());
        set_active_category.set("All".to_string());
        set_open.set(true);
    };

    view! {
        <>
            <div>
                <span class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1.5">"Icon"</span>
                <button
                    type="button"
                    on:click=move |_| open_picker()
                    class="w-full flex items-center gap-3 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 text-left"
                >
                    <span class="text-2xl leading-none">{move || emoji.get()}</span>
                    <span class="text-sm text-gray-500 dark:text-gray-400">
                        {move || if emoji.get() == DEFAULT_MARKER_EMOJI { "Pick an icon" } else { "Change icon" }}
                    </span>
                    <svg class="w-4 h-4 ml-auto text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                    </svg>
                </button>
            </div>

            {move || open.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div class="absolute inset-0 bg-black/60" on:click=move |_| set_open.set(false) />
                    <div class="relative w-full max-w-md bg-white dark:bg-gray-800 rounded-2xl shadow-2xl border border-gray-200 dark:border-gray-700 max-h-[80vh] flex flex-col">
                        <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                            <h3 class="text-sm font-semibold text-gray-900 dark:text-white">"Choose an icon"</h3>
                            <button
                                type="button"
                                on:click=move |_| set_open.set(false)
                                class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                                title="Close"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                                </svg>
                            </button>
                        </div>

                        <div class="px-4 pt-3">
                            <input
                                type="text"
                                placeholder="Search emojis..."
                                autocomplete="off"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white text-sm"
                                prop:value=query
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="flex flex-wrap gap-1.5 px-4 py-2">
                            {move || {
                                let mut cats = vec!["All".to_string()];
                                cats.extend(categories.get().into_iter().map(|c| c.name));
                                cats.into_iter().map(|name| {
                                    let is_active = active_category.get() == name;
                                    let name_clone = name.clone();
                                    view! {
                                        <button
                                            type="button"
                                            on:click=move |_| set_active_category.set(name_clone.clone())
                                            class=move || format!(
                                                "shrink-0 px-3 py-1.5 rounded-full text-sm font-medium transition-colors {}",
                                                if is_active {
                                                    "bg-indigo-600 text-white"
                                                } else {
                                                    "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
                                                }
                                            )
                                        >
                                            {name}
                                        </button>
                                    }
                                }).collect_view()
                            }}
                        </div>

                        <div class="grid grid-cols-8 gap-1 px-4 py-3 overflow-y-auto">
                            {move || filtered().into_iter().map(|entry| {
                                let entry_emoji = entry.emoji.clone();
                                let is_selected = emoji.get() == entry.emoji;
                                view! {
                                    <button
                                        type="button"
                                        on:click=move |_| {
                                            on_select.run(entry_emoji.clone());
                                            set_open.set(false);
                                        }
                                        class=move || format!(
                                            "aspect-square flex items-center justify-center rounded-lg text-xl transition-colors {}",
                                            if is_selected {
                                                "bg-indigo-100 dark:bg-indigo-900/40 ring-2 ring-indigo-500"
                                            } else {
                                                "hover:bg-gray-100 dark:hover:bg-gray-700"
                                            }
                                        )
                                        title=entry.title
                                    >
                                        {entry.emoji}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            })}
        </>
    }
}
