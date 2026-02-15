use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::features::{auth::UserSession, invites::models::InviteWithGroup};

#[server(GetInviteServer)]
async fn get_invite_server(uuid: String) -> Result<InviteWithGroup, ServerFnError> {
    crate::features::invites::handlers::get_invite_by_uuid(uuid).await
}

#[server(AcceptInviteServer)]
async fn accept_invite_server(uuid: String) -> Result<i64, ServerFnError> {
    crate::features::invites::handlers::accept_invite(uuid).await
}

/// Public invite accept page
#[must_use]
#[component]
pub fn InviteAccept() -> impl IntoView {
    let navigate = use_navigate();
    let params = use_params_map();
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();

    let uuid = Memo::new(move |_| params.read().get("uuid").unwrap_or_default());

    let invite_resource = LocalResource::new(move || get_invite_server(uuid.get()));
    let accept_action = ServerAction::<AcceptInviteServer>::new();

    // Check authentication and redirect to login if needed
    let navigate_for_auth = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            // User is not authenticated, redirect to login with return URL
            let current_path = format!("/invite/{}", uuid.get_untracked());
            let encoded_path = urlencoding::encode(&current_path);
            navigate_for_auth(
                &format!("/login?redirect_to={}", encoded_path),
                Default::default(),
            );
        }
    });

    // Store navigate in a StoredValue so it can be cloned
    let navigate_stored = StoredValue::new(navigate);

    // Effect to redirect after successful acceptance
    Effect::new(move |_| {
        if let Some(Ok(group_id)) = accept_action.value().get() {
            navigate_stored.with_value(|nav| {
                nav(&format!("/groups/{}", group_id), Default::default());
            });
        }
    });

    let on_accept = move |_| {
        accept_action.dispatch(AcceptInviteServer { uuid: uuid.get() });
    };

    let on_decline = move |_| {
        navigate_stored.with_value(|nav| {
            nav("/groups", Default::default());
        });
    };

    view! {
        <div class="min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center px-4">
            <div class="max-w-md w-full">
                <Suspense fallback=move || view! {
                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
                        <p class="mt-4 text-gray-600 dark:text-gray-400">"Loading invite..."</p>
                    </div>
                }>
                    {move || {
                        // Check if user is authenticated first
                        let is_authenticated = user_resource.get()
                            .and_then(|res| res.ok())
                            .and_then(|user| user)
                            .is_some();

                        // If not authenticated, show guest invite view
                        if !is_authenticated {
                            match invite_resource.get() {
                                Some(Ok(invite)) => {
                                    if !invite.is_valid {
                                        view! {
                                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                                                <div class="w-16 h-16 mx-auto mb-4 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center">
                                                    <svg class="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                                    </svg>
                                                </div>
                                                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"Invite Expired"</h2>
                                                <p class="text-gray-600 dark:text-gray-400 mb-6">"This invite has expired and can no longer be used."</p>
                                                <a
                                                    href="/"
                                                    class="inline-block px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-semibold rounded-lg"
                                                >
                                                    "Go Home"
                                                </a>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let current_path = format!("/invite/{}", uuid.get_untracked());
                                        let encoded_path = urlencoding::encode(&current_path);
                                        view! {
                                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8">
                                                <div class="text-center mb-6">
                                                    <div class="w-16 h-16 mx-auto mb-4 bg-indigo-100 dark:bg-indigo-900/30 rounded-full flex items-center justify-center">
                                                        <svg class="w-8 h-8 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                                        </svg>
                                                    </div>
                                                    <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"You're Invited!"</h2>
                                                    <p class="text-gray-600 dark:text-gray-400 mb-4">
                                                        "You've been invited to join "
                                                        <span class="font-semibold text-gray-900 dark:text-white">{invite.group_name}</span>
                                                    </p>
                                                    <p class="text-sm text-gray-500 dark:text-gray-400">
                                                        "Please login or register to accept this invitation."
                                                    </p>
                                                </div>

                                                <div class="flex flex-col gap-3">
                                                    <a
                                                        href={format!("/login?redirect_to={}", encoded_path)}
                                                        class="w-full px-6 py-3 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg text-center transition-all duration-200"
                                                    >
                                                        "Login to Accept"
                                                    </a>
                                                    <a
                                                        href={format!("/register?redirect_to={}", encoded_path)}
                                                        class="w-full px-6 py-3 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-white font-semibold rounded-lg text-center transition-all duration-200"
                                                    >
                                                        "Create Account"
                                                    </a>
                                                    <a
                                                        href="/"
                                                        class="text-center text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 mt-2"
                                                    >
                                                        "Cancel"
                                                    </a>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                },
                                Some(Err(e)) => view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                                        <div class="w-16 h-16 mx-auto mb-4 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center">
                                            <svg class="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                            </svg>
                                        </div>
                                        <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"Invite Not Found"</h2>
                                        <p class="text-gray-600 dark:text-gray-400 mb-6">{e.to_string()}</p>
                                        <a
                                            href="/"
                                            class="inline-block px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-semibold rounded-lg"
                                        >
                                            "Go Home"
                                        </a>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
                                    </div>
                                }.into_any()
                            }
                        } else {
                            // User is authenticated, show the accept/decline view
                            match invite_resource.get() {
                                Some(Ok(invite)) => {
                                    if !invite.is_valid {
                                        view! {
                                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                                                <div class="w-16 h-16 mx-auto mb-4 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center">
                                                    <svg class="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                                    </svg>
                                                </div>
                                                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"Invite Expired"</h2>
                                                <p class="text-gray-600 dark:text-gray-400 mb-6">"This invite has expired and can no longer be used."</p>
                                                <a
                                                    href="/groups"
                                                    class="inline-block px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-semibold rounded-lg"
                                                >
                                                    "Go to Groups"
                                                </a>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8">
                                                <div class="text-center mb-6">
                                                    <div class="w-16 h-16 mx-auto mb-4 bg-indigo-100 dark:bg-indigo-900/30 rounded-full flex items-center justify-center">
                                                        <svg class="w-8 h-8 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                                        </svg>
                                                    </div>
                                                    <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"You're Invited!"</h2>
                                                    <p class="text-gray-600 dark:text-gray-400">
                                                        "You've been invited to join "
                                                        <span class="font-semibold text-gray-900 dark:text-white">{invite.group_name}</span>
                                                    </p>
                                                </div>

                                                {move || {
                                                    accept_action.value().get().and_then(|result| {
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
                                                        on:click=on_accept
                                                        disabled=move || accept_action.pending().get()
                                                        class="flex-1 px-6 py-3 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-400 text-white font-semibold rounded-lg transition-all duration-200"
                                                    >
                                                        {move || if accept_action.pending().get() { "Accepting..." } else { "Accept Invitation" }}
                                                    </button>
                                                    <button
                                                        on:click=on_decline
                                                        class="px-6 py-3 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-white font-semibold rounded-lg transition-all duration-200"
                                                    >
                                                        "Decline"
                                                    </button>
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
                                },
                                Some(Err(e)) => view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                                        <div class="w-16 h-16 mx-auto mb-4 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center">
                                            <svg class="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                            </svg>
                                        </div>
                                        <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">"Invite Not Found"</h2>
                                        <p class="text-gray-600 dark:text-gray-400 mb-6">{e.to_string()}</p>
                                        <a
                                            href="/groups"
                                            class="inline-block px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-semibold rounded-lg"
                                        >
                                            "Go to Groups"
                                        </a>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8 text-center">
                                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
