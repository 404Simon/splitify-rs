use crate::features::auth::models::UserSession;
use crate::features::groups::handlers::get_group_members;
use crate::features::transactions::handlers::create_transaction;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

#[component]
pub fn TransactionsCreate() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let params = use_params_map();
    let group_id = move || {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
    };

    let members_resource = Resource::new(
        move || group_id(),
        |id| async move {
            match id {
                Some(group_id) => get_group_members(group_id).await,
                None => Err(ServerFnError::ServerError("Invalid group ID".to_string())),
            }
        },
    );

    let (recipient_id, set_recipient_id) = signal(0i64);
    let (amount, set_amount) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let (current_user_id, set_current_user_id) = signal(0i64);

    // Set current_user_id when user loads
    Effect::new(move |_| {
        if let Some(Ok(Some(user))) = user_resource.get() {
            set_current_user_id.set(user.id);
        }
    });

    let navigate = use_navigate();
    let create_action = Action::new(move |_: &()| {
        let navigate = navigate.clone();
        async move {
            let gid = group_id().unwrap_or(0);
            let rid = recipient_id.get();
            let amt = amount.get();
            let desc = description.get();

            if rid == 0 {
                set_error_message.set(Some("Please select a recipient".to_string()));
                return;
            }

            if amt.is_empty() {
                set_error_message.set(Some("Please enter an amount".to_string()));
                return;
            }

            let desc_opt = if desc.is_empty() { None } else { Some(desc) };

            match create_transaction(gid, rid, amt, desc_opt).await {
                Ok(_) => {
                    navigate(&format!("/groups/{}", gid), Default::default());
                }
                Err(e) => {
                    set_error_message.set(Some(e.to_string()));
                }
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        create_action.dispatch(());
    };

    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            {move || {
                match user_resource.get() {
                    Some(Ok(Some(_current_user))) => {
                        Either::Left(
                            view! {
                            <Suspense fallback=move || view! { <p>"Loading group..."</p> }>
                                {move || {
                                    group_id()
                                        .map(|_gid| {
                                                    view! {
                                                        <div class="min-h-screen bg-gray-100 dark:bg-gray-900 py-6">
                                                            <div class="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8">
                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">"Add Transaction"</h1>
                                                            <form on:submit=on_submit class="space-y-4">
                                                                {move || {
                                                                    error_message
                                                                        .get()
                                                                        .map(|msg| {
                                                                            view! {
                                                                                <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                                                                                    <p class="text-red-800 dark:text-red-200">{msg}</p>
                                                                                </div>
                                                                            }
                                                                        })
                                                                }}

                                                                <div>
                                                                    <label
                                                                        for="recipient_id"
                                                                        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                                                                    >
                                                                        "Recipient"
                                                                    </label>
                                                                    <Suspense fallback=move || {
                                                                        view! { <p>"Loading members..."</p> }
                                                                    }>
                                                                        {move || {
                                                                            members_resource
                                                                                .get()
                                                                                .map(|members_result| match members_result {
                                                                                     Ok(members) => {
                                                                                        let other_members: Vec<_> = members
                                                                                            .into_iter()
                                                                                            .filter(|m| m.id != current_user_id.get())
                                                                                            .collect();
                                                                                        view! {
                                                                                            <select
                                                                                                id="recipient_id"
                                                                                                on:change=move |ev| {
                                                                                                    let value = event_target_value(&ev);
                                                                                                    if let Ok(id) = value.parse::<i64>() {
                                                                                                        set_recipient_id.set(id);
                                                                                                    }
                                                                                                }

                                                                                                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white"
                                                                                            >
                                                                                                <option value="0">"Select Recipient"</option>
                                                                                                {other_members
                                                                                                    .into_iter()
                                                                                                    .map(|member| {
                                                                                                        view! {
                                                                                                            <option value=member.id>
                                                                                                                {member.username}
                                                                                                            </option>
                                                                                                        }
                                                                                                    })
                                                                                                    .collect_view()}
                                                                                            </select>
                                                                                        }
                                                                                        .into_any()
                                                                                    }
                                                                                    Err(e) => {
                                                                                        view! {
                                                                                            <p class="text-red-600">{e.to_string()}</p>
                                                                                        }
                                                                                        .into_any()
                                                                                    }
                                                                                })
                                                                        }}

                                                                    </Suspense>
                                                                </div>

                                                                <div>
                                                                    <label
                                                                        for="amount"
                                                                        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                                                                    >
                                                                        "Amount (€)"
                                                                    </label>
                                                                    <input
                                                                        type="number"
                                                                        id="amount"
                                                                        step="0.01"
                                                                        min="0.01"
                                                                        required
                                                                        placeholder="0.00"
                                                                        on:input=move |ev| {
                                                                            set_amount.set(event_target_value(&ev));
                                                                        }

                                                                        prop:value=amount
                                                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white"
                                                                    />
                                                                </div>

                                                                <div>
                                                                    <label
                                                                        for="description"
                                                                        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                                                                    >
                                                                        "Description (optional)"
                                                                    </label>
                                                                    <input
                                                                        type="text"
                                                                        id="description"
                                                                        placeholder="e.g., Dinner payment"
                                                                        on:input=move |ev| {
                                                                            set_description.set(event_target_value(&ev));
                                                                        }

                                                                        prop:value=description
                                                                        class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white"
                                                                    />
                                                                </div>

                                                                <button
                                                                    type="submit"
                                                                    class="w-full px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium transition-colors"
                                                                >
                                                                    "Add Transaction"
                                                                </button>
                                                            </form>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                })
                                        }}

                                    </Suspense>
                                },
                            )
                    }
                    Some(Ok(None)) => {
                        let navigate = use_navigate();
                        Effect::new(move |_| {
                            navigate("/login", Default::default());
                        });
                        Either::Right(view! { <p>"Redirecting..."</p> })
                    }
                    Some(Err(_)) => {
                        Either::Right(view! { <p>"Error loading user session"</p> })
                    }
                    None => {
                        Either::Right(view! { <p>"Loading..."</p> })
                    }
                }
            }}

        </Suspense>
    }
}
