use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::{
    components::{AppLayout, Navigation},
    features::{
        auth::{use_logout, UserSession},
        groups::handlers::get_group,
        invites::{
            handlers::{get_group_invites, CreateInvite, DeleteInvite},
            models::InviteListItem,
        },
    },
};

/// Groups invites page - manage invites for a group
#[must_use]
#[component]
pub fn GroupsInvites() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let create_invite_action = ServerAction::<CreateInvite>::new();
    let delete_invite_action = ServerAction::<DeleteInvite>::new();
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

    let invites_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_invites(id).await }
    });

    // Form signals
    let name_signal = RwSignal::new(String::new());
    let duration_days_signal = RwSignal::new(7_i64);
    let is_reusable_signal = RwSignal::new(false);

    // Effect to redirect if not authenticated
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate("/login", Default::default());
        }
    });

    // Effect to refetch invites after creation
    Effect::new(move |_| {
        if let Some(Ok(_)) = create_invite_action.value().get() {
            invites_resource.refetch();
            // Reset form
            name_signal.set(String::new());
            duration_days_signal.set(7);
            is_reusable_signal.set(false);
        }
    });

    // Effect to refetch invites after deletion
    Effect::new(move |_| {
        if let Some(Ok(_)) = delete_invite_action.value().get() {
            invites_resource.refetch();
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        create_invite_action.dispatch(CreateInvite {
            group_id: group_id.get(),
            name: if name_signal.get().is_empty() {
                None
            } else {
                Some(name_signal.get())
            },
            duration_days: duration_days_signal.get(),
            is_reusable: is_reusable_signal.get(),
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
                                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                            {move || {
                                                match group_resource.get() {
                                                    Some(Ok(group)) => {
                                                        let is_admin = group.created_by == user.id;

                                                        if !is_admin {
                                                            return view! {
                                                                <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                                    <p class="text-sm text-red-700 dark:text-red-300">"Only group admins can manage invites"</p>
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
                                                                            "Manage Invites for " {group.name.clone()}
                                                                        </h1>
                                                                    </div>
                                                                </div>

                                                                // Create Invite Form
                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
                                                                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Create New Invite"</h2>

                                                                    <form on:submit=on_submit class="space-y-4">
                                                                        <div>
                                                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                                                "Invite Name (Optional)"
                                                                            </label>
                                                                            <input
                                                                                type="text"
                                                                                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-indigo-500"
                                                                                placeholder="e.g., For John"
                                                                                on:input=move |ev| name_signal.set(event_target_value(&ev))
                                                                                prop:value=move || name_signal.get()
                                                                            />
                                                                        </div>

                                                                        <div>
                                                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                                                "Duration (Days)"
                                                                            </label>
                                                                            <input
                                                                                type="number"
                                                                                min="1"
                                                                                max="30"
                                                                                required
                                                                                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-indigo-500"
                                                                                on:input=move |ev| {
                                                                                    if let Ok(val) = event_target_value(&ev).parse::<i64>() {
                                                                                        duration_days_signal.set(val);
                                                                                    }
                                                                                }
                                                                                prop:value=move || duration_days_signal.get()
                                                                            />
                                                                        </div>

                                                                        <div class="flex items-center">
                                                                            <input
                                                                                type="checkbox"
                                                                                id="is_reusable"
                                                                                class="w-4 h-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
                                                                                on:change=move |ev| is_reusable_signal.set(event_target_checked(&ev))
                                                                                prop:checked=move || is_reusable_signal.get()
                                                                            />
                                                                            <label for="is_reusable" class="ml-2 text-sm text-gray-700 dark:text-gray-300">
                                                                                "Reusable (can be used multiple times)"
                                                                            </label>
                                                                        </div>

                                                                        <button
                                                                            type="submit"
                                                                            disabled=move || create_invite_action.pending().get()
                                                                            class="w-full sm:w-auto px-6 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-400 text-white font-semibold rounded-lg transition-colors"
                                                                        >
                                                                            {move || if create_invite_action.pending().get() { "Creating..." } else { "Create Invite" }}
                                                                        </button>
                                                                    </form>

                                                                    {move || {
                                                                        create_invite_action.value().get().map(|result| {
                                                                            match result {
                                                                                Ok(_) => view! {
                                                                                    <div class="mt-4 rounded-md bg-green-50 dark:bg-green-900/30 p-4">
                                                                                        <p class="text-sm text-green-700 dark:text-green-300">"Invite created successfully!"</p>
                                                                                    </div>
                                                                                }.into_any(),
                                                                                Err(e) => view! {
                                                                                    <div class="mt-4 rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                                                        <p class="text-sm text-red-700 dark:text-red-300">{e.to_string()}</p>
                                                                                    </div>
                                                                                }.into_any()
                                                                            }
                                                                        })
                                                                    }}
                                                                </div>

                                                                // Invites List
                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Pending Invites"</h2>

                                                                    <Suspense fallback=move || view! { <div>"Loading invites..."</div> }>
                                                                        {move || {
                                                                            match invites_resource.get() {
                                                                                Some(Ok(invites)) => {
                                                                                    if invites.is_empty() {
                                                                                        view! {
                                                                                            <p class="text-gray-500 dark:text-gray-400">"No invites yet. Create one above!"</p>
                                                                                        }.into_any()
                                                                                    } else {
                                                                                        view! {
                                                                                            <div class="space-y-4">
                                                                                                {invites.into_iter().map(|invite: InviteListItem| {
                                                                                                    let invite_uuid = invite.uuid.clone();
                                                                                                    let invite_uuid_for_delete = invite.uuid.clone();
                                                                                                    let group_id_val = group_id.get();

                                                                                                    view! {
                                                                                                        <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                                                                                                            <div class="flex flex-col sm:flex-row sm:justify-between sm:items-start gap-3">
                                                                                                                <div class="flex-1">
                                                                                                                    <div class="flex items-center gap-2 mb-1">
                                                                                                                        <span class="font-semibold text-gray-900 dark:text-white">
                                                                                                                            {invite.name.clone().unwrap_or_else(|| "Invite".to_string())}
                                                                                                                        </span>
                                                                                                                        {if invite.is_reusable {
                                                                                                                            view! {
                                                                                                                                <span class="px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 text-xs font-medium rounded">
                                                                                                                                    "Reusable"
                                                                                                                                </span>
                                                                                                                            }.into_any()
                                                                                                                        } else {
                                                                                                                            view! {
                                                                                                                                <span class="px-2 py-1 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300 text-xs font-medium rounded">
                                                                                                                                    "Single Use"
                                                                                                                                </span>
                                                                                                                            }.into_any()
                                                                                                                        }}
                                                                                                                    </div>
                                                                                                                    <p class="text-sm text-gray-600 dark:text-gray-400">
                                                                                                                        "Expires: " {invite.expiration_date}
                                                                                                                    </p>
                                                                                                    <div class="mt-2 bg-gray-50 dark:bg-gray-700 rounded p-2">
                                                                                                        <code class="text-xs text-gray-800 dark:text-gray-200 break-all select-all">
                                                                                                            {window().location().origin().map(|origin| format!("{}/invite/{}", origin, invite_uuid)).unwrap_or_else(|_| format!("/invite/{}", invite_uuid))}
                                                                                                        </code>
                                                                                                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                                                                                            "Click text to select, then Ctrl+C to copy"
                                                                                                        </p>
                                                                                                    </div>
                                                                                                                </div>
                                                                                                                <button
                                                                                                                    class="px-4 py-2 bg-red-100 hover:bg-red-200 dark:bg-red-900/30 dark:hover:bg-red-900/50 text-red-700 dark:text-red-300 rounded-lg font-medium transition-colors"
                                                                                                                    on:click=move |_| {
                                                                                                                        delete_invite_action.dispatch(DeleteInvite {
                                                                                                                            uuid: invite_uuid_for_delete.clone(),
                                                                                                                            group_id: group_id_val,
                                                                                                                        });
                                                                                                                    }
                                                                                                                >
                                                                                                                    "Delete"
                                                                                                                </button>
                                                                                                            </div>
                                                                                                        </div>
                                                                                                    }
                                                                                                }).collect_view()}
                                                                                            </div>
                                                                                        }.into_any()
                                                                                    }
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
