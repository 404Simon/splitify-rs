use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
};

use crate::{
    components::{
        AppLayout, ErrorAlert, FormActions, FormCard, FormField, LoadingSpinner, Navigation,
        PageHeader,
    },
    features::{
        auth::{use_logout, UserSession},
        groups::handlers::get_group,
        shopping_lists::CreateShoppingList,
    },
};

/// Create shopping list page
#[must_use]
#[component]
pub fn ShoppingListCreate() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let navigate = use_navigate();
    let on_logout = use_logout();
    let params = use_params_map();

    let group_id = Memo::new(move |_| {
        params
            .read()
            .get("group_id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    });

    let group_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group(id).await }
    });

    let create_action = ServerAction::<CreateShoppingList>::new();
    let (name, set_name) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Effect to redirect if not authenticated
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate_clone("/login", Default::default());
        }
    });

    // Effect to handle submission result
    Effect::new(move |_| {
        if let Some(result) = create_action.value().get() {
            match result {
                Ok(list_id) => {
                    navigate(
                        &format!("/groups/{}/shopping-lists/{}", group_id.get(), list_id),
                        Default::default(),
                    );
                }
                Err(e) => {
                    set_error_message.set(Some(e.to_string()));
                }
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(None);

        create_action.dispatch(CreateShoppingList {
            group_id: group_id.get(),
            name: name.get(),
        });
    };

    let gid = group_id.get_untracked();

    view! {
        <Suspense fallback=LoadingSpinner>
            {move || {
                match user_resource.get() {
                    Some(Ok(Some(user))) => view! {
                        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
                            <Navigation username=user.username.clone() on_logout=on_logout />
                            <AppLayout>
                                <div class="py-6">
                                    <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                            {move || {
                                                match group_resource.get() {
                                                    Some(Ok(group)) => view! {
                                                        <A href=format!("/groups/{}", gid) attr:class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-900 dark:hover:text-indigo-300 text-sm inline-flex items-center mb-3">
                                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                                                            </svg>
                                                            "Back to Group"
                                                        </A>

                                                        <PageHeader title=format!("Create Shopping List for {}", group.name) />

                                                        <FormCard>
                                                            <form on:submit=on_submit class="space-y-6">
                                                                <ErrorAlert message=error_message />

                                                                <FormField label="List Name" for_id="name">
                                                                    <input
                                                                        type="text"
                                                                        id="name"
                                                                        required
                                                                        maxlength="255"
                                                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white"
                                                                        placeholder="e.g., Weekly Groceries"
                                                                        value=name.get_untracked()
                                                                        on:input=move |ev| set_name.set(event_target_value(&ev))
                                                                    />
                                                                </FormField>

                                                                <FormActions
                                                                    submit_text="Create List"
                                                                    loading_text="Creating..."
                                                                    loading=Signal::derive(move || create_action.pending().get())
                                                                    cancel_href=format!("/groups/{}", gid)
                                                                />
                                                            </form>
                                                        </FormCard>
                                                    }.into_any(),
                                                    Some(Err(e)) => view! {
                                                        <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                            <p class="text-sm text-red-700 dark:text-red-300">"Error: " {e.to_string()}</p>
                                                        </div>
                                                    }.into_any(),
                                                    None => view! { <div>"Loading..."</div> }.into_any()
                                                }
                                            }}
                                        </Suspense>
                                    </div>
                                </div>
                            </AppLayout>
                        </div>
                    }.into_any(),
                    _ => LoadingSpinner().into_any()
                }
            }}
        </Suspense>
    }
}
