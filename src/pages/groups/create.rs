use crate::components::{AppLayout, Navigation};
use crate::features::auth::{use_logout, UserSession};
use crate::features::groups::handlers::CreateGroup;
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Create group page
#[component]
pub fn GroupsCreate() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let create_action = ServerAction::<CreateGroup>::new();
    let navigate = use_navigate();
    let on_logout = use_logout();

    let (group_name, set_group_name) = signal(String::new());

    // Clone navigate for use in multiple effects
    let navigate_clone = navigate.clone();

    // Effect to redirect after successful creation
    Effect::new(move |_| {
        if let Some(Ok(group_id)) = create_action.value().get() {
            navigate_clone(&format!("/groups/{}", group_id), Default::default());
        }
    });

    // Effect to redirect if not authenticated
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate("/login", Default::default());
        }
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let name = group_name.get();
        if !name.trim().is_empty() {
            create_action.dispatch(CreateGroup { name });
        }
    };

    view! {
        <Suspense fallback=move || view! {
            <div class="flex justify-center items-center min-h-screen bg-gray-100 dark:bg-gray-900">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
            </div>
        }>
            {move || {
                match user_resource.get() {
                    Some(Ok(Some(user))) => view! {
                        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
                            <Navigation username=user.username.clone() on_logout=on_logout />
                            <AppLayout>
                                <div class="py-6">
                                    <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <div class="mb-8">
                                            <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">"Create New Group"</h1>
                                            <p class="text-gray-600 dark:text-gray-400 mt-1">"Start a new expense group"</p>
                                        </div>

                                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                            <form on:submit=on_submit>
                                                <div class="mb-6">
                                                    <label
                                                        for="group-name"
                                                        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                                                    >
                                                        "Group Name"
                                                    </label>
                                                    <input
                                                        type="text"
                                                        id="group-name"
                                                        name="name"
                                                        required
                                                        prop:value=group_name
                                                        on:input=move |ev| set_group_name.set(event_target_value(&ev))
                                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white"
                                                        placeholder="e.g., Weekend Trip, Roommates, Project Team"
                                                    />
                                                </div>

                                                {move || {
                                                    create_action.value().get().and_then(|result| {
                                                        match result {
                                                            Ok(_) => None,
                                                            Err(e) => Some(view! {
                                                                <div class="mb-4 rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                                    <p class="text-sm text-red-700 dark:text-red-300">{e.to_string()}</p>
                                                                </div>
                                                            }.into_any())
                                                        }
                                                    })
                                                }}

                                                <div class="flex gap-3">
                                                    <button
                                                        type="submit"
                                                        disabled=move || create_action.pending().get()
                                                        class="flex-1 px-6 py-3 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-400 text-white font-semibold rounded-lg shadow-md hover:shadow-lg transition-all duration-200"
                                                    >
                                                        {move || if create_action.pending().get() { "Creating..." } else { "Create Group" }}
                                                    </button>
                                                    <a
                                                        href="/groups"
                                                        class="px-6 py-3 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-white font-semibold rounded-lg transition-all duration-200"
                                                    >
                                                        "Cancel"
                                                    </a>
                                                </div>
                                            </form>
                                        </div>
                                    </div>
                                </div>
                            </AppLayout>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="flex justify-center items-center min-h-screen bg-gray-100 dark:bg-gray-900">
                            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                        </div>
                    }.into_any()
                }
            }}
        </Suspense>
    }
}
