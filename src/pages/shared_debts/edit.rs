use crate::components::{AppLayout, Navigation};
use crate::features::auth::{use_logout, UserSession};
use crate::features::groups::handlers::get_group_members;
use crate::features::shared_debts::handlers::{
    get_shared_debt, get_shared_debt_members, UpdateSharedDebt,
};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

/// Edit shared debt page
#[component]
pub fn SharedDebtsEdit() -> impl IntoView {
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

    let debt_id = Memo::new(move |_| {
        params
            .read()
            .get("debt_id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    });

    let debt_resource = LocalResource::new(move || {
        let id = debt_id.get();
        async move { get_shared_debt(id).await }
    });

    let members_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_members(id).await }
    });

    let debt_members_resource = LocalResource::new(move || {
        let id = debt_id.get();
        async move { get_shared_debt_members(id).await }
    });

    let update_action = ServerAction::<UpdateSharedDebt>::new();
    let (name, set_name) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (selected_members, set_selected_members) = signal(Vec::<i64>::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Effect to redirect if not authenticated
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate_clone("/login", Default::default());
        }
    });

    // Effect to populate form when debt loads
    Effect::new(move |_| {
        if let Some(Ok(debt)) = debt_resource.get() {
            set_name.set(debt.name.clone());
            set_amount.set(debt.amount.to_string());
        }
    });

    // Effect to populate selected members
    Effect::new(move |_| {
        if let Some(Ok(member_ids)) = debt_members_resource.get() {
            set_selected_members.set(member_ids);
        }
    });

    // Effect to handle submission result
    Effect::new(move |_| {
        if let Some(result) = update_action.value().get() {
            match result {
                Ok(_) => {
                    navigate(&format!("/groups/{}", group_id.get()), Default::default());
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

        update_action.dispatch(UpdateSharedDebt {
            debt_id: debt_id.get(),
            name: name.get(),
            amount: amount.get(),
            member_ids: selected_members.get(),
        });
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
                                        <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                            {move || {
                                                match debt_resource.get() {
                                                    Some(Ok(_debt)) if !_debt.is_creator => view! {
                                                        <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                            <p class="text-sm text-red-700 dark:text-red-300">
                                                                "You do not have permission to edit this debt."
                                                            </p>
                                                            <a
                                                                href=format!("/groups/{}", group_id.get())
                                                                class="text-sm text-red-700 dark:text-red-300 underline mt-2 inline-block"
                                                            >
                                                                "Back to Group"
                                                            </a>
                                                        </div>
                                                    }.into_any(),
                                                    Some(Ok(_debt)) => view! {
                                                        <div class="mb-8">
                                                            <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">
                                                                "Edit Debt"
                                                            </h1>
                                                        </div>

                                                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                            <form on:submit=on_submit class="space-y-6">
                                                                {move || error_message.get().map(|msg| view! {
                                                                    <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                                        <p class="text-sm text-red-700 dark:text-red-300">{msg}</p>
                                                                    </div>
                                                                })}

                                                                <div>
                                                                    <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                                        "Name"
                                                                    </label>
                                                                    <input
                                                                        type="text"
                                                                        id="name"
                                                                        required
                                                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white"
                                                                        prop:value=move || name.get()
                                                                        on:input=move |ev| set_name.set(event_target_value(&ev))
                                                                    />
                                                                </div>

                                                                <div>
                                                                    <label for="amount" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                                        "Amount (€)"
                                                                    </label>
                                                                    <input
                                                                        type="number"
                                                                        id="amount"
                                                                        required
                                                                        step="0.01"
                                                                        min="0.01"
                                                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white"
                                                                        prop:value=move || amount.get()
                                                                        on:input=move |ev| set_amount.set(event_target_value(&ev))
                                                                    />
                                                                </div>

                                                                <div>
                                                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                                        "Split Between"
                                                                    </label>
                                                                    <Suspense fallback=move || view! { <div>"Loading members..."</div> }>
                                                                        {move || {
                                                                            match members_resource.get() {
                                                                                Some(Ok(members)) => {
                                                                                    let selected = selected_members.get();
                                                                                    view! {
                                                                                        <div class="space-y-2 max-h-64 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
                                                                                            {members.into_iter().map(|member| {
                                                                                                let member_id = member.id;
                                                                                                let is_checked = selected.contains(&member_id);
                                                                                                view! {
                                                                                                    <div class="flex items-center">
                                                                                                        <input
                                                                                                            type="checkbox"
                                                                                                            id=format!("member-{}", member_id)
                                                                                                            checked=is_checked
                                                                                                            class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 dark:border-gray-700 rounded bg-white dark:bg-gray-700"
                                                                                                            on:change=move |ev| {
                                                                                                                let checked = event_target_checked(&ev);
                                                                                                                set_selected_members.update(|members| {
                                                                                                                    if checked {
                                                                                                                        if !members.contains(&member_id) {
                                                                                                                            members.push(member_id);
                                                                                                                        }
                                                                                                                    } else {
                                                                                                                        members.retain(|&id| id != member_id);
                                                                                                                    }
                                                                                                                });
                                                                                                            }
                                                                                                        />
                                                                                                        <label
                                                                                                            for=format!("member-{}", member_id)
                                                                                                            class="ml-2 text-gray-700 dark:text-gray-300"
                                                                                                        >
                                                                                                            {member.username}
                                                                                                        </label>
                                                                                                    </div>
                                                                                                }
                                                                                            }).collect_view()}
                                                                                        </div>
                                                                                    }.into_any()
                                                                                },
                                                                                Some(Err(e)) => view! {
                                                                                    <div class="text-red-600 dark:text-red-400">"Error: " {e.to_string()}</div>
                                                                                }.into_any(),
                                                                                None => view! { <div>"Loading..."</div> }.into_any()
                                                                            }
                                                                        }}
                                                                    </Suspense>
                                                                </div>

                                                                <div class="flex gap-3">
                                                                    <button
                                                                        type="submit"
                                                                        disabled=move || update_action.pending().get()
                                                                        class="flex-1 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors"
                                                                    >
                                                                        {move || if update_action.pending().get() { "Updating..." } else { "Update Debt" }}
                                                                    </button>
                                                                    <a
                                                                        href=format!("/groups/{}", group_id.get())
                                                                        class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium transition-colors"
                                                                    >
                                                                        "Cancel"
                                                                    </a>
                                                                </div>
                                                            </form>
                                                        </div>
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
