use crate::components::{AppLayout, Navigation};
use crate::features::auth::{use_logout, UserSession};
use crate::features::groups::handlers::{
    get_all_users, get_group, get_group_members, DeleteGroup, UpdateGroup,
};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

/// Groups edit page - edit group name and members
#[component]
pub fn GroupsEdit() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let update_group_action = ServerAction::<UpdateGroup>::new();
    let delete_group_action = ServerAction::<DeleteGroup>::new();
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

    let group_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group(id).await }
    });

    let members_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_members(id).await }
    });

    let all_users_resource = LocalResource::new(|| async move { get_all_users().await });

    // Form signals
    let name_signal = RwSignal::new(String::new());
    let selected_members = RwSignal::new(Vec::<i64>::new());
    let show_delete_modal = RwSignal::new(false);

    // Effect to redirect if not authenticated
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate_clone("/login", Default::default());
        }
    });

    // Effect to populate form when group loads
    Effect::new(move |_| {
        if let Some(Ok(group)) = group_resource.get() {
            name_signal.set(group.name.clone());
        }
    });

    // Effect to populate selected members when members load
    Effect::new(move |_| {
        if let Some(Ok(members)) = members_resource.get() {
            selected_members.set(members.iter().map(|m| m.id).collect());
        }
    });

    // Effect to redirect after successful update
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(_)) = update_group_action.value().get() {
            let id = group_id.get();
            navigate_clone(&format!("/groups/{id}"), Default::default());
        }
    });

    // Effect to redirect after successful deletion
    Effect::new(move |_| {
        if let Some(Ok(_)) = delete_group_action.value().get() {
            navigate("/groups", Default::default());
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        update_group_action.dispatch(UpdateGroup {
            group_id: group_id.get(),
            name: name_signal.get(),
            member_ids: selected_members.get(),
        });
    };

    let on_delete_confirm = move |_| {
        delete_group_action.dispatch(DeleteGroup {
            group_id: group_id.get(),
        });
        show_delete_modal.set(false);
    };

    view! {
        <Suspense fallback=move || view! {
            <div class="flex justify-center items-center min-h-screen bg-gray-100 dark:bg-gray-900">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
            </div>
        }>
            {move || {
                match user_resource.get() {
                    Some(Ok(Some(user))) => {
                        let user_id = user.id;
                        let username = StoredValue::new(user.username.clone());

                        view! {
                            <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
                                <Navigation username=username.get_value() on_logout=on_logout />
                                <AppLayout>
                                    <div class="py-6">
                                        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                                            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                                {move || {
                                                    match group_resource.get() {
                                                        Some(Ok(group)) => {
                                                            let is_admin = group.created_by == user_id;

                                                        if !is_admin {
                                                            return view! {
                                                                <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                                    <p class="text-sm text-red-700 dark:text-red-300">"Only group admins can edit groups"</p>
                                                                </div>
                                                            }.into_any();
                                                        }

                                                        view! {
                                                            <div>
                                                                // Header
                                                                <div class="mb-8">
                                                                    <div class="flex items-center gap-2 mb-2">
                                                                        <a href=format!("/groups/{}", group.id) class="text-indigo-600 hover:text-indigo-700 dark:text-indigo-400">
                                                                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
                                                                            </svg>
                                                                        </a>
                                                                        <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">
                                                                            "Edit " {group.name.clone()}
                                                                        </h1>
                                                                    </div>
                                                                </div>

                                                                // Edit Form
                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
                                                                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Group Details"</h2>

                                                                    <form on:submit=on_submit class="space-y-6">
                                                                        <div>
                                                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                                                "Group Name"
                                                                            </label>
                                                                            <input
                                                                                type="text"
                                                                                required
                                                                                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-indigo-500"
                                                                                placeholder="e.g., Roommates"
                                                                                on:input=move |ev| name_signal.set(event_target_value(&ev))
                                                                                prop:value=move || name_signal.get()
                                                                            />
                                                                        </div>

                                                                        <div>
                                                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                                                "Members"
                                                                            </label>
                                                                            <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">
                                                                                "Select users to add to this group. Hold Ctrl/Cmd to select multiple."
                                                                            </p>

                                                                            <Suspense fallback=move || view! { <div>"Loading users..."</div> }>
                                                                                {move || {
                                                                                    match (all_users_resource.get(), members_resource.get()) {
                                                                                        (Some(Ok(all_users)), Some(Ok(current_members))) => {
                                                                                            // Combine current user with all users to create full list
                                                                                            let mut available_users = vec![UserSession {
                                                                                                id: user_id,
                                                                                                username: username.get_value()
                                                                                            }];
                                                                                            available_users.extend(all_users);
                                                                                            available_users.sort_by(|a, b| a.username.cmp(&b.username));
                                                                                            available_users.dedup_by(|a, b| a.id == b.id);

                                                                                            let current_user_id = user_id;
                                                                                            let is_creator = current_members.iter().any(|m| m.id == current_user_id && m.is_creator);

                                                                                            view! {
                                                                                                <div class="border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 max-h-64 overflow-y-auto">
                                                                                                    {available_users.into_iter().map(|u| {
                                                                                                        let user_id = u.id;
                                                                                                        let is_current_user = user_id == current_user_id;
                                                                                                        let is_user_creator = current_members.iter().any(|m| m.id == user_id && m.is_creator);
                                                                                                        let disabled = is_current_user && is_creator;

                                                                                                        view! {
                                                                                                            <label class="flex items-center px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-600 cursor-pointer border-b border-gray-200 dark:border-gray-600 last:border-0">
                                                                                                                <input
                                                                                                                    type="checkbox"
                                                                                                                    class="w-4 h-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
                                                                                                                    disabled=disabled
                                                                                                                    prop:checked=move || selected_members.get().contains(&user_id)
                                                                                                                    on:change=move |ev| {
                                                                                                                        let checked = event_target_checked(&ev);
                                                                                                                        selected_members.update(|members| {
                                                                                                                            if checked {
                                                                                                                                if !members.contains(&user_id) {
                                                                                                                                    members.push(user_id);
                                                                                                                                }
                                                                                                                            } else {
                                                                                                                                members.retain(|&id| id != user_id);
                                                                                                                            }
                                                                                                                        });
                                                                                                                    }
                                                                                                                />
                                                                                                                <span class="ml-3 text-gray-900 dark:text-white">
                                                                                                                    {u.username}
                                                                                                                    {if is_current_user { " (You)" } else { "" }}
                                                                                                                    {if is_user_creator { " - Creator" } else { "" }}
                                                                                                                </span>
                                                                                                            </label>
                                                                                                        }
                                                                                                    }).collect_view()}
                                                                                                </div>
                                                                                                {if is_creator {
                                                                                                    view! {
                                                                                                        <p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                                                                                                            "Note: You cannot remove yourself as the group creator."
                                                                                                        </p>
                                                                                                    }.into_any()
                                                                                                } else {
                                                                                                    ().into_any()
                                                                                                }}
                                                                                            }.into_any()
                                                                                        },
                                                                                        _ => view! { <div>"Loading..."</div> }.into_any()
                                                                                    }
                                                                                }}
                                                                            </Suspense>
                                                                        </div>

                                                                        <div class="flex flex-col sm:flex-row gap-3">
                                                                            <button
                                                                                type="submit"
                                                                                disabled=move || update_group_action.pending().get()
                                                                                class="px-6 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-400 text-white font-semibold rounded-lg transition-colors"
                                                                            >
                                                                                {move || if update_group_action.pending().get() { "Saving..." } else { "Save Changes" }}
                                                                            </button>
                                                                            <a
                                                                                href=format!("/groups/{}", group.id)
                                                                                class="px-6 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white font-semibold rounded-lg transition-colors text-center"
                                                                            >
                                                                                "Cancel"
                                                                            </a>
                                                                        </div>
                                                                    </form>

                                                                    {move || {
                                                                        update_group_action.value().get().and_then(|result| {
                                                                            match result {
                                                                                Ok(_) => None, // Redirect happens via Effect
                                                                                Err(e) => Some(view! {
                                                                                    <div class="mt-4 rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                                                        <p class="text-sm text-red-700 dark:text-red-300">{e.to_string()}</p>
                                                                                    </div>
                                                                                }.into_any())
                                                                            }
                                                                        })
                                                                    }}
                                                                </div>

                                                                // Danger Zone
                                                                <div class="bg-red-50 dark:bg-red-900/20 rounded-xl shadow-sm border border-red-200 dark:border-red-800 p-6">
                                                                    <h2 class="text-lg font-semibold text-red-900 dark:text-red-200 mb-2">"Danger Zone"</h2>
                                                                    <p class="text-sm text-red-700 dark:text-red-300 mb-4">
                                                                        "Deleting this group will permanently remove all associated data, including expenses, transactions, and debts. This action cannot be undone."
                                                                    </p>
                                                                    <button
                                                                        type="button"
                                                                        on:click=move |_| show_delete_modal.set(true)
                                                                        class="px-6 py-2 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors"
                                                                    >
                                                                        "Delete Group"
                                                                    </button>

                                                                    {move || {
                                                                        delete_group_action.value().get().and_then(|result| {
                                                                            match result {
                                                                                Ok(_) => None, // Redirect happens via Effect
                                                                                Err(e) => Some(view! {
                                                                                    <div class="mt-4 rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                                                        <p class="text-sm text-red-700 dark:text-red-300">{e.to_string()}</p>
                                                                                    </div>
                                                                                }.into_any())
                                                                            }
                                                                        })
                                                                    }}
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    },
                                                    Some(Err(e)) => view! {
                                                        <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                            <p class="text-sm text-red-700 dark:text-red-300">{e.to_string()}</p>
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
                    }.into_any()
                    },
                    _ => view! {
                        <div class="flex justify-center items-center min-h-screen bg-gray-100 dark:bg-gray-900">
                            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                        </div>
                    }.into_any()
                }
            }}
        </Suspense>

        // Delete Confirmation Modal
        {move || {
            if show_delete_modal.get() {
                view! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-w-md w-full p-6">
                            <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-3">
                                "Confirm Deletion"
                            </h3>
                            <p class="text-gray-700 dark:text-gray-300 mb-6">
                                "Are you sure you want to delete this group? This action cannot be undone and will permanently delete all associated data."
                            </p>
                            <div class="flex gap-3">
                                <button
                                    on:click=on_delete_confirm
                                    disabled=move || delete_group_action.pending().get()
                                    class="flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-gray-400 text-white font-semibold rounded-lg transition-colors"
                                >
                                    {move || if delete_group_action.pending().get() { "Deleting..." } else { "Yes, Delete" }}
                                </button>
                                <button
                                    on:click=move |_| show_delete_modal.set(false)
                                    class="flex-1 px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white font-semibold rounded-lg transition-colors"
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }
        }}
    }
}
