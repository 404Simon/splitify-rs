use leptos::{ev, prelude::*};
use leptos_router::{components::A, hooks::use_navigate};

use crate::{
    components::{
        forms::{
            CancelButton, ErrorAlert, FormCard, FormField, FormInput, LoadingSpinner, PageHeader,
            SubmitButton,
        },
        AppLayout, Navigation,
    },
    features::{
        auth::{use_logout, UserSession},
        groups::handlers::CreateGroup,
    },
};

/// Create group page
#[must_use]
#[component]
pub fn GroupsCreate() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let create_action = ServerAction::<CreateGroup>::new();
    let navigate = use_navigate();
    let on_logout = use_logout();

    let (group_name, set_group_name) = signal(String::new());
    let (error_message, set_error_message) = signal(None::<String>);

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

    // Effect to handle errors
    Effect::new(move |_| {
        if let Some(result) = create_action.value().get() {
            match result {
                Ok(_) => set_error_message.set(None),
                Err(e) => set_error_message.set(Some(e.to_string())),
            }
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
        <Suspense fallback=move || view! { <LoadingSpinner /> }>
            {move || {
                match user_resource.get() {
                    Some(Ok(Some(user))) => view! {
                        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
                            <Navigation username=user.username.clone() on_logout=on_logout />
                            <AppLayout>
                                <div class="py-6">
                                    <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <A href="/groups" attr:class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-900 dark:hover:text-indigo-300 text-sm inline-flex items-center mb-3">
                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                                            </svg>
                                            "Back to Groups"
                                        </A>

                                        <PageHeader
                                            title="Create New Group".to_string()
                                            subtitle="Start a new expense group".to_string()
                                        />

                                        <FormCard>
                                            <form on:submit=on_submit class="space-y-6">
                                                <FormField label="Group Name" for_id="group-name">
                                                    <FormInput
                                                        id="group-name"
                                                        input_type="text"
                                                        placeholder="e.g., Weekend Trip, Roommates, Project Team"
                                                        value=Signal::derive(move || group_name.get())
                                                        on_input=Callback::new(move |val| set_group_name.set(val))
                                                        required=true
                                                    />
                                                </FormField>

                                                <ErrorAlert message=error_message />

                                                <div class="flex gap-3">
                                                    <SubmitButton
                                                        text="Create Group"
                                                        loading_text="Creating..."
                                                        loading=Signal::derive(move || create_action.pending().get())
                                                    />
                                                    <CancelButton href="/groups".to_string() />
                                                </div>
                                            </form>
                                        </FormCard>
                                    </div>
                                </div>
                            </AppLayout>
                        </div>
                    }.into_any(),
                    _ => view! { <LoadingSpinner /> }.into_any()
                }
            }}
        </Suspense>
    }
}
