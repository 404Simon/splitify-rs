use crate::components::{AppLayout, Navigation};
use crate::features::auth::{use_logout, UserSession};
use crate::features::recurring_debts::handlers::{
    get_generated_instances, get_recurring_debt, get_recurring_debt_shares, DeleteRecurringDebt,
    GenerateNow, ToggleRecurringDebtActive,
};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

/// Show recurring debt details page
#[component]
pub fn RecurringDebtsShow() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let navigate = use_navigate();
    let on_logout = use_logout();
    let params = use_params_map();

    let group_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    });

    let recurring_id = Memo::new(move |_| {
        params
            .read()
            .get("recurring_id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    });

    let debt_resource = LocalResource::new(move || {
        let id = recurring_id.get();
        async move { get_recurring_debt(id).await }
    });

    let shares_resource = LocalResource::new(move || {
        let id = recurring_id.get();
        async move { get_recurring_debt_shares(id).await }
    });

    let instances_resource = LocalResource::new(move || {
        let id = recurring_id.get();
        async move { get_generated_instances(id).await }
    });

    let toggle_action = ServerAction::<ToggleRecurringDebtActive>::new();
    let generate_action = ServerAction::<GenerateNow>::new();
    let delete_action = ServerAction::<DeleteRecurringDebt>::new();

    // Effect to redirect if not authenticated
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate_clone("/login", Default::default());
        }
    });

    // Effect to reload debt when toggle completes
    Effect::new(move |_| {
        if let Some(Ok(_)) = toggle_action.value().get() {
            debt_resource.refetch();
        }
    });

    // Effect to reload instances when generate completes
    Effect::new(move |_| {
        if let Some(Ok(_)) = generate_action.value().get() {
            instances_resource.refetch();
            debt_resource.refetch();
        }
    });

    // Effect to redirect when delete completes
    Effect::new(move |_| {
        if let Some(Ok(_)) = delete_action.value().get() {
            navigate(&format!("/groups/{}", group_id.get()), Default::default());
        }
    });

    let on_toggle = move |_| {
        toggle_action.dispatch(ToggleRecurringDebtActive {
            recurring_debt_id: recurring_id.get(),
        });
    };

    let on_generate = move |_| {
        generate_action.dispatch(GenerateNow {
            recurring_debt_id: recurring_id.get(),
        });
    };

    let on_delete = move |_| {
        if window()
            .confirm_with_message("Are you sure you want to delete this recurring debt? Generated debts will remain, but no new ones will be created.")
            .unwrap_or(false)
        {
            delete_action.dispatch(DeleteRecurringDebt {
                recurring_debt_id: recurring_id.get(),
            });
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
                                    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                            {move || {
                                                match debt_resource.get() {
                                                    Some(Ok(debt)) => view! {
                                                        <div class="mb-8">
                                                            <div class="flex items-center justify-between">
                                                                <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">
                                                                    {debt.name.clone()}
                                                                </h1>
                                                                <span class={format!(
                                                                    "px-3 py-1 rounded-full text-sm font-medium {}",
                                                                    if debt.is_active {
                                                                        "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                                                                    } else {
                                                                        "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400"
                                                                    }
                                                                )}>
                                                                    {if debt.is_active { "Active" } else { "Paused" }}
                                                                </span>
                                                            </div>
                                                            <a
                                                                href=format!("/groups/{}", group_id.get())
                                                                class="text-indigo-600 hover:text-indigo-800 dark:text-indigo-400 text-sm mt-2 inline-block"
                                                            >
                                                                "← Back to Group"
                                                            </a>
                                                        </div>

                                                        // Overview Card
                                                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
                                                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                                                                "Overview"
                                                            </h2>
                                                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                                                <div>
                                                                    <p class="text-sm text-gray-500 dark:text-gray-400">"Amount"</p>
                                                                    <p class="text-xl font-semibold text-gray-900 dark:text-white">
                                                                        "€" {debt.amount.to_string()}
                                                                    </p>
                                                                </div>
                                                                <div>
                                                                    <p class="text-sm text-gray-500 dark:text-gray-400">"Frequency"</p>
                                                                    <p class="text-xl font-semibold text-gray-900 dark:text-white capitalize">
                                                                        {debt.frequency.to_string()}
                                                                    </p>
                                                                </div>
                                                                <div>
                                                                    <p class="text-sm text-gray-500 dark:text-gray-400">"Start Date"</p>
                                                                    <p class="text-lg text-gray-900 dark:text-white">
                                                                        {debt.start_date.to_string()}
                                                                    </p>
                                                                </div>
                                                                <div>
                                                                    <p class="text-sm text-gray-500 dark:text-gray-400">"End Date"</p>
                                                                    <p class="text-lg text-gray-900 dark:text-white">
                                                                        {debt.end_date.map(|d| d.to_string()).unwrap_or_else(|| "Never".to_string())}
                                                                    </p>
                                                                </div>
                                                                <div>
                                                                    <p class="text-sm text-gray-500 dark:text-gray-400">"Next Generation"</p>
                                                                    <p class="text-lg text-gray-900 dark:text-white">
                                                                        {debt.next_generation_date.to_string()}
                                                                    </p>
                                                                </div>
                                                                <div>
                                                                    <p class="text-sm text-gray-500 dark:text-gray-400">"Created By"</p>
                                                                    <p class="text-lg text-gray-900 dark:text-white">
                                                                        {debt.creator_username.clone()}
                                                                    </p>
                                                                </div>
                                                            </div>
                                                        </div>

                                                        // Members & Shares Card
                                                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
                                                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                                                                "Members & Shares"
                                                            </h2>
                                                            <Suspense fallback=move || view! { <div>"Loading shares..."</div> }>
                                                                {move || {
                                                                    match shares_resource.get() {
                                                                        Some(Ok(shares)) => view! {
                                                                            <div class="space-y-2">
                                                                                {shares.into_iter().map(|share| {
                                                                                    view! {
                                                                                        <div class="flex justify-between items-center py-2 border-b border-gray-100 dark:border-gray-700 last:border-0">
                                                                                            <span class="text-gray-900 dark:text-white">
                                                                                                {share.username}
                                                                                            </span>
                                                                                            <span class="font-medium text-gray-900 dark:text-white">
                                                                                                "€" {share.share_amount.to_string()}
                                                                                            </span>
                                                                                        </div>
                                                                                    }
                                                                                }).collect_view()}
                                                                            </div>
                                                                        }.into_any(),
                                                                        Some(Err(e)) => view! {
                                                                            <div class="text-red-600 dark:text-red-400">"Error: " {e.to_string()}</div>
                                                                        }.into_any(),
                                                                        None => view! { <div>"Loading..."</div> }.into_any()
                                                                    }
                                                                }}
                                                            </Suspense>
                                                        </div>

                                                        // Generated Instances Card
                                                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
                                                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                                                                "Generated Debts History"
                                                            </h2>
                                                            <Suspense fallback=move || view! { <div>"Loading history..."</div> }>
                                                                {move || {
                                                                    match instances_resource.get() {
                                                                        Some(Ok(instances)) if instances.is_empty() => view! {
                                                                            <p class="text-gray-500 dark:text-gray-400 text-sm">
                                                                                "No debts have been generated yet."
                                                                            </p>
                                                                        }.into_any(),
                                                                        Some(Ok(instances)) => view! {
                                                                            <div class="space-y-2 max-h-96 overflow-y-auto">
                                                                                {instances.into_iter().map(|instance| {
                                                                                    view! {
                                                                                        <div class="flex justify-between items-center py-2 border-b border-gray-100 dark:border-gray-700 last:border-0">
                                                                                            <div>
                                                                                                <p class="text-gray-900 dark:text-white font-medium">
                                                                                                    {instance.debt_name}
                                                                                                </p>
                                                                                                <p class="text-sm text-gray-500 dark:text-gray-400">
                                                                                                    "Created: " {instance.created_at.date().to_string()}
                                                                                                </p>
                                                                                            </div>
                                                                                            <span class="font-medium text-gray-900 dark:text-white">
                                                                                                "€" {instance.amount.to_string()}
                                                                                            </span>
                                                                                        </div>
                                                                                    }
                                                                                }).collect_view()}
                                                                            </div>
                                                                        }.into_any(),
                                                                        Some(Err(e)) => view! {
                                                                            <div class="text-red-600 dark:text-red-400">"Error: " {e.to_string()}</div>
                                                                        }.into_any(),
                                                                        None => view! { <div>"Loading..."</div> }.into_any()
                                                                    }
                                                                }}
                                                            </Suspense>
                                                        </div>

                                                        // Actions
                                                        {if debt.is_creator {
                                                            view! {
                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                                                                        "Actions"
                                                                    </h2>
                                                                    <div class="flex flex-wrap gap-3">
                                                                        <button
                                                                            on:click=on_generate
                                                                            disabled=move || generate_action.pending().get()
                                                                            class="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors"
                                                                        >
                                                                            {move || if generate_action.pending().get() { "Generating..." } else { "Generate Now" }}
                                                                        </button>
                                                                        <button
                                                                            on:click=on_toggle
                                                                            disabled=move || toggle_action.pending().get()
                                                                            class="px-4 py-2 bg-yellow-600 hover:bg-yellow-700 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors"
                                                                        >
                                                                            {move || {
                                                                                if toggle_action.pending().get() {
                                                                                    "Processing...".to_string()
                                                                                } else if debt.is_active {
                                                                                    "Pause".to_string()
                                                                                } else {
                                                                                    "Resume".to_string()
                                                                                }
                                                                            }}
                                                                        </button>
                                                                        <a
                                                                            href=format!("/groups/{}/recurring-debts/{}/edit", group_id.get(), recurring_id.get())
                                                                            class="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium transition-colors"
                                                                        >
                                                                            "Edit"
                                                                        </a>
                                                                        <button
                                                                            on:click=on_delete
                                                                            disabled=move || delete_action.pending().get()
                                                                            class="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors"
                                                                        >
                                                                            {move || if delete_action.pending().get() { "Deleting..." } else { "Delete" }}
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="bg-yellow-50 dark:bg-yellow-900/30 rounded-lg p-4">
                                                                    <p class="text-sm text-yellow-700 dark:text-yellow-300">
                                                                        "Only the creator can modify this recurring debt."
                                                                    </p>
                                                                </div>
                                                            }.into_any()
                                                        }}
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
