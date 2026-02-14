use leptos::prelude::*;

use crate::features::shopping_lists::{get_shopping_lists, ShoppingListSummary};

#[component]
pub fn ShoppingListsSection(group_id: Memo<i64>) -> impl IntoView {
    let lists_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_shopping_lists(id).await }
    });

    view! {
        <div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">"Shopping Lists"</h2>
                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                        "Collaborative shopping lists for your group"
                    </p>
                </div>
                <a
                    href=move || format!("/groups/{}/shopping-lists/create", group_id.get())
                    class="inline-flex items-center px-3 py-2 text-sm font-medium rounded-lg text-white bg-indigo-600 hover:bg-indigo-700 transition-colors"
                >
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    "New List"
                </a>
            </div>

            <Suspense fallback=move || view! {
                <div class="flex justify-center py-8">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                </div>
            }>
                {move || {
                    match lists_resource.get() {
                        Some(Ok(lists)) if lists.is_empty() => view! {
                            <div class="text-center py-8 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                <svg class="w-12 h-12 mx-auto text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                                </svg>
                                <p class="text-gray-600 dark:text-gray-400 text-sm">"No shopping lists yet. Create your first one!"</p>
                            </div>
                        }.into_any(),
                        Some(Ok(lists)) => view! {
                            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                                {lists.into_iter().map(|list| view! { <ShoppingListCard list=list group_id=group_id /> }).collect_view()}
                            </div>
                        }.into_any(),
                        Some(Err(e)) => view! {
                            <div class="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg p-4">
                                <div class="flex items-start">
                                    <svg class="w-5 h-5 text-red-600 dark:text-red-400 mt-0.5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                    </svg>
                                    <div>
                                        <h3 class="text-sm font-medium text-red-800 dark:text-red-300">"Error loading shopping lists"</h3>
                                        <p class="mt-1 text-sm text-red-700 dark:text-red-400">{e.to_string()}</p>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        None => view! {
                            <div class="flex justify-center py-8">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn ShoppingListCard(list: ShoppingListSummary, group_id: Memo<i64>) -> impl IntoView {
    let progress = list.completion_percentage();

    view! {
        <a
            href=format!("/groups/{}/shopping-lists/{}", group_id.get(), list.id)
            class="block bg-gray-50 dark:bg-gray-700/50 rounded-lg border border-gray-200 dark:border-gray-600 hover:border-indigo-300 dark:hover:border-indigo-600 hover:shadow-md transition-all p-4"
        >
            <div class="flex items-start justify-between mb-2">
                <div class="flex-1 min-w-0">
                    <h3 class="text-base font-semibold text-gray-900 dark:text-white truncate">{list.name}</h3>
                    <p class="text-xs text-gray-600 dark:text-gray-400 mt-0.5">
                        "by " {list.creator_username}
                    </p>
                </div>
                {(list.total_items > 0).then(|| view! {
                    <span class="ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-indigo-100 dark:bg-indigo-900/50 text-indigo-800 dark:text-indigo-300">
                        {list.completed_items} "/" {list.total_items}
                    </span>
                })}
            </div>

            {(list.total_items > 0).then(|| view! {
                <div class="mt-3">
                    <div class="flex items-center justify-between text-xs text-gray-600 dark:text-gray-400 mb-1.5">
                        <span>"Progress"</span>
                        <span class="font-medium">{format!("{:.0}%", progress)}</span>
                    </div>
                    <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-1.5">
                        <div
                            class="bg-indigo-600 dark:bg-indigo-500 h-1.5 rounded-full transition-all"
                            style:width=format!("{}%", progress)
                        />
                    </div>
                </div>
            })}

            {(list.total_items == 0).then(|| view! {
                <p class="mt-2 text-xs text-gray-500 dark:text-gray-400 italic">"No items yet"</p>
            })}
        </a>
    }
}
