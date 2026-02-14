use leptos::prelude::{ServerFnError, *};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
};

use crate::features::shopping_lists::{get_shopping_list, UpdateShoppingList};

#[component]
pub fn ShoppingListEdit() -> impl IntoView {
    let params = use_params_map();
    let (group_id, list_id) = (
        move || {
            params
                .read()
                .get("group_id")
                .and_then(|id| id.parse::<i64>().ok())
        },
        move || {
            params
                .read()
                .get("list_id")
                .and_then(|id| id.parse::<i64>().ok())
        },
    );

    #[allow(clippy::redundant_closure)]
    let list_resource = Resource::new(
        move || list_id(),
        |id| async move {
            match id {
                Some(id) => get_shopping_list(id).await,
                None => Err(ServerFnError::new("Missing list_id")),
            }
        },
    );

    let (name, set_name) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let update_action = ServerAction::<UpdateShoppingList>::new();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(list)) = list_resource.get() {
            set_name.set(list.name);
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = update_action.value().get() {
            if let (Some(gid), Some(lid)) = (group_id(), list_id()) {
                let url = format!("/groups/{}/shopping-lists/{}", gid, lid);
                navigate(&url, Default::default());
            }
        } else if let Some(Err(e)) = update_action.value().get() {
            set_error.set(Some(e.to_string()));
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        if let Some(lid) = list_id() {
            update_action.dispatch(UpdateShoppingList {
                list_id: lid,
                name: name.get(),
            });
        }
    };

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                <A href=move || format!("/groups/{}/shopping-lists/{}", group_id().unwrap_or(0), list_id().unwrap_or(0)) attr:class="text-indigo-600 hover:text-indigo-900 text-sm mb-4 inline-block">
                    "← Back to List"
                </A>

                <h1 class="text-3xl font-bold text-gray-900 mb-8">"Edit Shopping List"</h1>

                <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                    {move || Suspend::new(async move {
                        match list_resource.await {
                            Ok(_list) => {
                                view! {
                                    <div class="bg-white rounded-lg shadow p-6">
                                        <form on:submit=on_submit class="space-y-6">
                                            <div>
                                                <label for="name" class="block text-sm font-medium text-gray-700 mb-2">
                                                    "List Name"
                                                </label>
                                                <input
                                                    type="text"
                                                    id="name"
                                                    name="name"
                                                    prop:value=name
                                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                                    required
                                                    maxlength="255"
                                                    class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                                                />
                                            </div>

                                            <Show when=move || error.get().is_some()>
                                                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                                                    <p class="text-red-800 text-sm">{move || error.get()}</p>
                                                </div>
                                            </Show>

                                            <div class="flex gap-3 justify-end">
                                                <A
                                                    href=move || format!("/groups/{}/shopping-lists/{}", group_id().unwrap_or(0), list_id().unwrap_or(0))
                                                    attr:class="px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 dark:bg-gray-900"
                                                >
                                                    "Cancel"
                                                </A>
                                                <button
                                                    type="submit"
                                                    disabled=move || update_action.pending().get()
                                                    class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
                                                >
                                                    {move || if update_action.pending().get() { "Saving..." } else { "Save Changes" }}
                                                </button>
                                            </div>
                                        </form>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                                    <p class="text-red-800">"Error: " {e.to_string()}</p>
                                </div>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
