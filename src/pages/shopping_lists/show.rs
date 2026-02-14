use leptos::prelude::{ServerFnError, *};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    components::{InputLabel, PrimaryButton, TextInput},
    features::shopping_lists::{
        get_shopping_list, get_shopping_list_activity, get_shopping_list_items,
        AddShoppingListItem, DeleteShoppingList, DeleteShoppingListItem, ShoppingListActivity,
        ShoppingListItem, ToggleShoppingListItem,
    },
};

#[component]
pub fn ShoppingListShow() -> impl IntoView {
    let params = use_params_map();
    #[cfg(feature = "hydrate")]
    let group_id = move || {
        params
            .read()
            .get("group_id")
            .and_then(|id| id.parse::<i64>().ok())
    };
    let list_id = move || {
        params
            .read()
            .get("list_id")
            .and_then(|id| id.parse::<i64>().ok())
    };

    let list_resource = LocalResource::new(move || {
        let id = list_id();
        async move {
            match id {
                Some(id) => get_shopping_list(id).await,
                None => Err(ServerFnError::new("Missing list_id")),
            }
        }
    });

    let items_resource = LocalResource::new(move || {
        let id = list_id();
        async move {
            match id {
                Some(id) => get_shopping_list_items(id).await,
                None => Err(ServerFnError::new("Missing list_id")),
            }
        }
    });

    let activity_resource = LocalResource::new(move || {
        let id = list_id();
        async move {
            match id {
                Some(id) => get_shopping_list_activity(id, 10).await,
                None => Err(ServerFnError::new("Missing list_id")),
            }
        }
    });

    // SSE connection for real-time updates
    #[cfg(feature = "hydrate")]
    {
        use leptos::web_sys::{EventSource, MessageEvent};
        use wasm_bindgen::{prelude::*, JsCast};

        Effect::new(move |_| {
            if let (Some(gid), Some(lid)) = (group_id(), list_id()) {
                let url = format!("/api/groups/{}/shopping-lists/{}/events", gid, lid);

                if let Ok(es) = EventSource::new(&url) {
                    let items_resource_clone = items_resource;
                    let activity_resource_clone = activity_resource;

                    let on_message = Closure::wrap(Box::new(move |_: MessageEvent| {
                        items_resource_clone.refetch();
                        activity_resource_clone.refetch();
                    })
                        as Box<dyn FnMut(MessageEvent)>);

                    es.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                    on_message.forget();

                    // Leak EventSource to keep connection alive
                    Box::leak(Box::new(es));
                }
            }
        });
    }

    let (show_completed, set_show_completed) = signal(true);
    let item_name = RwSignal::new(String::new());
    let item_quantity = RwSignal::new(String::new());
    let item_category = RwSignal::new(String::new());
    let (show_delete_modal, set_show_delete_modal) = signal(false);

    let add_item_action = ServerAction::<AddShoppingListItem>::new();
    let toggle_item_action = ServerAction::<ToggleShoppingListItem>::new();
    let delete_item_action = ServerAction::<DeleteShoppingListItem>::new();
    let delete_list_action = ServerAction::<DeleteShoppingList>::new();

    // Reset form after successful add
    Effect::new(move |_| {
        if let Some(Ok(_)) = add_item_action.value().get() {
            item_name.set(String::new());
            item_quantity.set(String::new());
            item_category.set(String::new());
        }
    });

    let on_add_item = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(lid) = list_id() {
            let qty = item_quantity.get();
            let cat = item_category.get();
            add_item_action.dispatch(AddShoppingListItem {
                list_id: lid,
                name: item_name.get(),
                quantity: if qty.is_empty() { None } else { Some(qty) },
                category: if cat.is_empty() { None } else { Some(cat) },
            });
        }
    };

    view! {
        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                <Suspense fallback=move || view! {
                    <div class="flex justify-center items-center py-12">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                    </div>
                }>
                    {move || {
                        match list_resource.get() {
                            Some(Ok(list)) => {
                                let gid = list.group_id;
                                let list_name = list.name.clone();

                                view! {
                                    <div>
                                        <div class="mb-6">
                                            <A href=format!("/groups/{}", gid) attr:class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-900 dark:hover:text-indigo-300 text-sm inline-flex items-center mb-3">
                                                <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                                                </svg>
                                                "Back to Group"
                                            </A>
                                            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                                                <div>
                                                    <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">{list_name.clone()}</h1>
                                                    <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">"Shopping List Details"</p>
                                                </div>
                                                <div class="flex gap-2">
                                                    <A
                                                        href=format!("/groups/{}/shopping-lists/{}/edit", gid, list.id)
                                                        attr:class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium transition-colors inline-flex items-center"
                                                    >
                                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                                                        </svg>
                                                        "Edit"
                                                    </A>
                                                    <button
                                                        on:click=move |_| set_show_delete_modal.set(true)
                                                        class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors inline-flex items-center"
                                                    >
                                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                                        </svg>
                                                        "Delete"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                                            <div class="lg:col-span-2">
                                                <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 mb-6">
                                                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Add Item"</h2>
                                                    <form on:submit=on_add_item class="space-y-4">
                                                        <div>
                                                            <InputLabel for_input="item_name">"Item Name"</InputLabel>
                                                            <TextInput
                                                                input_type="text"
                                                                placeholder="Enter item name"
                                                                required=true
                                                                class="w-full mt-1"
                                                                value=item_name
                                                            />
                                                        </div>
                                                        <div class="grid grid-cols-2 gap-4">
                                                            <div>
                                                                <InputLabel for_input="item_quantity">"Quantity (optional)"</InputLabel>
                                                                <TextInput
                                                                    input_type="text"
                                                                    placeholder="2 kg"
                                                                    class="w-full mt-1"
                                                                    value=item_quantity
                                                                />
                                                            </div>
                                                            <div>
                                                                <InputLabel for_input="item_category">"Category (optional)"</InputLabel>
                                                                <TextInput
                                                                    input_type="text"
                                                                    placeholder="e.g. Produce"
                                                                    class="w-full mt-1"
                                                                    value=item_category
                                                                />
                                                            </div>
                                                        </div>
                                                        <PrimaryButton
                                                            button_type="submit"
                                                            disabled=Signal::derive(move || add_item_action.pending().get())
                                                        >
                                                            {move || if add_item_action.pending().get() { "Adding..." } else { "Add Item" }}
                                                        </PrimaryButton>
                                                    </form>
                                                </div>

                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                                                    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
                                                        <div class="flex items-center justify-between">
                                                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Items"</h2>
                                                            <button
                                                                on:click=move |_| set_show_completed.update(|v| *v = !*v)
                                                                class="text-sm text-indigo-600 dark:text-indigo-400 hover:text-indigo-900 dark:hover:text-indigo-300 font-medium"
                                                            >
                                                                {move || if show_completed.get() { "Hide completed" } else { "Show completed" }}
                                                            </button>
                                                        </div>
                                                    </div>

                                                    <Suspense fallback=move || view! { <div class="p-6 text-center text-gray-500 dark:text-gray-400">"Loading items..."</div> }>
                                                        {move || {
                                                            match items_resource.get() {
                                                                Some(Ok(items)) => {
                                                                    let filtered_items: Vec<_> = items.into_iter()
                                                                        .filter(|item| show_completed.get() || !item.is_completed)
                                                                        .collect();

                                                                    if filtered_items.is_empty() {
                                                                        view! {
                                                                            <div class="p-12 text-center">
                                                                                <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center">
                                                                                    <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
                                                                                    </svg>
                                                                                </div>
                                                                                <p class="text-gray-500 dark:text-gray-400">"No items yet. Add some above!"</p>
                                                                            </div>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <ul class="divide-y divide-gray-200 dark:divide-gray-700">
                                                                                {filtered_items.into_iter().map(|item| {
                                                                                    view! { <ItemRow item=item toggle_action=toggle_item_action delete_action=delete_item_action /> }
                                                                                }).collect_view()}
                                                                            </ul>
                                                                        }.into_any()
                                                                    }
                                                                }
                                                                Some(Err(e)) => view! {
                                                                    <div class="p-6 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg m-6">
                                                                        <p class="text-red-800 dark:text-red-300">"Error: " {e.to_string()}</p>
                                                                    </div>
                                                                }.into_any(),
                                                                None => view! {
                                                                    <div class="p-6 text-center text-gray-500 dark:text-gray-400">"Loading items..."</div>
                                                                }.into_any()
                                                            }
                                                        }}
                                                    </Suspense>
                                                </div>
                                            </div>

                                            <div class="lg:col-span-1">
                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Recent Activity"</h2>
                                                    <Suspense fallback=move || view! {
                                                        <div class="text-center py-4">
                                                            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600 mx-auto"></div>
                                                        </div>
                                                    }>
                                                        {move || {
                                                            match activity_resource.get() {
                                                                Some(Ok(activities)) => {
                                                                    if activities.is_empty() {
                                                                        view! {
                                                                            <p class="text-sm text-gray-500 dark:text-gray-400 italic text-center py-4">"No activity yet"</p>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <ul class="space-y-3">
                                                                                {activities.into_iter().map(|activity| {
                                                                                    view! { <ActivityItem activity /> }
                                                                                }).collect_view()}
                                                                            </ul>
                                                                        }.into_any()
                                                                    }
                                                                }
                                                                Some(Err(_)) => view! {
                                                                    <p class="text-sm text-red-600 dark:text-red-400">"Failed to load activity"</p>
                                                                }.into_any(),
                                                                None => view! {
                                                                    <div class="text-center py-4">
                                                                        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600 mx-auto"></div>
                                                                    </div>
                                                                }.into_any()
                                                            }
                                                        }}
                                                    </Suspense>
                                                </div>
                                            </div>
                                        </div>

                                        <DeleteModal
                                            show=show_delete_modal
                                            on_confirm=Callback::new(move |_| {
                                                if let Some(lid) = list_id() {
                                                    delete_list_action.dispatch(DeleteShoppingList { list_id: lid });
                                                }
                                            })
                                            on_cancel=Callback::new(move |_| set_show_delete_modal.set(false))
                                            list_name=list_name.clone()
                                        />
                                    </div>
                                }.into_any()
                            }
                            Some(Err(e)) => view! {
                                <div class="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-xl p-6">
                                    <div class="flex items-start">
                                        <svg class="w-5 h-5 text-red-600 dark:text-red-400 mt-0.5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                        </svg>
                                        <div>
                                            <h3 class="text-sm font-medium text-red-800 dark:text-red-300">"Error loading shopping list"</h3>
                                            <p class="mt-1 text-sm text-red-700 dark:text-red-400">{e.to_string()}</p>
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),
                            None => view! {
                                <div class="flex justify-center items-center py-12">
                                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                                </div>
                            }.into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn ItemRow(
    item: ShoppingListItem,
    toggle_action: ServerAction<ToggleShoppingListItem>,
    delete_action: ServerAction<DeleteShoppingListItem>,
) -> impl IntoView {
    let item_id = item.id;
    let name = item.name.clone();
    let quantity = item.quantity.clone();
    let category = item.category.clone();
    let completed_by_username = item.completed_by_username.clone();
    let is_completed = item.is_completed;

    view! {
        <li class="p-4 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
            <div class="flex items-center gap-4">
                <input
                    type="checkbox"
                    prop:checked=is_completed
                    on:change=move |_| { toggle_action.dispatch(ToggleShoppingListItem { item_id }); }
                    class="h-5 w-5 rounded border-gray-300 dark:border-gray-600 text-indigo-600 focus:ring-indigo-500 dark:bg-gray-800"
                />
                <div class="flex-1 min-w-0">
                    <p class=if is_completed {
                        "text-gray-500 dark:text-gray-400 line-through"
                    } else {
                        "text-gray-900 dark:text-white font-medium"
                    }>
                        {name.clone()}
                        {quantity.as_ref().map(|q| format!(" ({})", q))}
                    </p>
                    {category.as_ref().map(|c| view! {
                        <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 mt-1">
                            {c.clone()}
                        </span>
                    })}
                    {completed_by_username.as_ref().map(|username| view! {
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                            "Completed by " {username.clone()}
                        </p>
                    })}
                </div>
                <button
                    on:click=move |_| { delete_action.dispatch(DeleteShoppingListItem { item_id }); }
                    class="text-red-600 dark:text-red-400 hover:text-red-900 dark:hover:text-red-300 p-2 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                    title="Delete item"
                >
                    <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                </button>
            </div>
        </li>
    }
}

#[component]
fn ActivityItem(activity: ShoppingListActivity) -> impl IntoView {
    view! {
        <li class="text-sm border-l-2 border-indigo-500 pl-3">
            <p class="text-gray-900 dark:text-white font-medium">{activity.action_description()}</p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">{activity.created_at.to_string()}</p>
        </li>
    }
}

#[component]
fn DeleteModal(
    show: ReadSignal<bool>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
    list_name: String,
) -> impl IntoView {
    let list_name = StoredValue::new(list_name);
    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 z-50 overflow-y-auto">
                <div class="flex items-center justify-center min-h-screen px-4">
                    <div class="fixed inset-0 bg-gray-900 bg-opacity-75 transition-opacity" on:click=move |_| on_cancel.run(())></div>
                    <div class="relative bg-white dark:bg-gray-800 rounded-xl shadow-xl p-6 max-w-md w-full border border-gray-200 dark:border-gray-700">
                        <div class="mb-4">
                            <div class="w-12 h-12 mx-auto mb-4 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center">
                                <svg class="w-6 h-6 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                                </svg>
                            </div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white text-center mb-2">"Delete Shopping List"</h3>
                            <p class="text-sm text-gray-600 dark:text-gray-400 text-center">
                                "Are you sure you want to delete \"" <span class="font-medium">{list_name.get_value()}</span> "\"? This action cannot be undone."
                            </p>
                        </div>
                        <div class="flex gap-3 justify-end">
                            <button
                                on:click=move |_| on_cancel.run(())
                                class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium transition-colors"
                            >
                                "Cancel"
                            </button>
                            <button
                                on:click=move |_| on_confirm.run(())
                                class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors"
                            >
                                "Delete"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
