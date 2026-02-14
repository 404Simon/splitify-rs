use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::{
    components::{GuestLayout, InputLabel, PrimaryButton, TextInput},
    features::auth::{LoginUser, UserSession},
};

/// Login page component
#[must_use]
#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<LoginUser>::new();
    let username_signal = RwSignal::new(String::new());
    let password_signal = RwSignal::new(String::new());
    let navigate = use_navigate();
    let query_map = use_query_map();
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();

    // Get redirect_to query parameter
    let redirect_to = Memo::new(move |_| {
        query_map
            .read()
            .get("redirect_to")
            .and_then(|encoded| urlencoding::decode(&encoded).ok().map(|s| s.into_owned()))
    });

    // Redirect logged-in users
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(Some(_))) = user_resource.get() {
            let target = redirect_to.get().unwrap_or_else(|| "/groups".to_string());
            navigate_clone(&target, Default::default());
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let username = username_signal.get();
        let password = password_signal.get();
        login_action.dispatch(LoginUser { username, password });
    };

    // Effect to handle navigation after successful login
    let has_refetched = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(Ok(_)) = login_action.value().get() {
            if !has_refetched.get() {
                // Refetch user resource to update the global user context
                user_resource.refetch();
                has_refetched.set(true);
            }
        }
    });

    // Separate effect to navigate after resource is updated
    Effect::new(move |_| {
        if has_refetched.get() {
            if let Some(Ok(Some(_))) = user_resource.get() {
                let target = redirect_to.get().unwrap_or_else(|| "/groups".to_string());
                navigate(&target, Default::default());
            }
        }
    });

    view! {
        <GuestLayout>
            <div>
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">"Login"</h2>

                {move || {
                    redirect_to.get().and_then(|path| {
                        if path.contains("/invite/") {
                            Some(view! {
                                <div class="mb-4 rounded-md bg-indigo-50 dark:bg-indigo-900/30 p-4">
                                    <p class="text-sm text-indigo-700 dark:text-indigo-300">
                                        "Please login to accept your group invitation."
                                    </p>
                                </div>
                            })
                        } else {
                            None
                        }
                    })
                }}

                <form on:submit=on_submit class="space-y-6">
                    <div>
                        <InputLabel for_input="username">"Username"</InputLabel>
                        <TextInput
                            input_type="text"
                            class="block mt-1 w-full"
                            required=true
                            value=username_signal
                        />
                    </div>

                    <div>
                        <InputLabel for_input="password">"Password"</InputLabel>
                        <TextInput
                            input_type="password"
                            class="block mt-1 w-full"
                            required=true
                            value=password_signal
                        />
                    </div>

                    <div class="flex items-center justify-end">
                        <PrimaryButton
                            button_type="submit"
                            disabled=login_action.pending()
                        >
                            {move || if login_action.pending().get() { "Logging in..." } else { "Log in" }}
                        </PrimaryButton>
                    </div>

                    {move || {
                        login_action.value().get().map(|result| {
                            match result {
                                Ok(_) => {
                                    view! {
                                        <div class="rounded-md bg-green-50 dark:bg-green-900/30 p-4">
                                            <p class="text-sm text-green-700 dark:text-green-300">"Login successful! Redirecting..."</p>
                                        </div>
                                    }.into_any()
                                },
                                Err(e) => view! {
                                    <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                        <p class="text-sm text-red-700 dark:text-red-300">{e.to_string()}</p>
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </form>

                <div class="mt-6 text-center">
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                        "Don't have an account? "
                        <a
                            href={move || {
                                redirect_to.get()
                                    .map(|path| format!("/register?redirect_to={}", urlencoding::encode(&path)))
                                    .unwrap_or_else(|| "/register".to_string())
                            }}
                            class="font-medium text-indigo-600 hover:text-indigo-500 dark:text-indigo-400 dark:hover:text-indigo-300"
                        >
                            "Register here"
                        </a>
                    </p>
                </div>
            </div>
        </GuestLayout>
    }
}
