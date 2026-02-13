use crate::components::{GuestLayout, InputLabel, PrimaryButton, TextInput};
use crate::features::auth::{RegisterUser, UserSession};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Registration page component
#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_action = ServerAction::<RegisterUser>::new();
    let username_signal = RwSignal::new(String::new());
    let password_signal = RwSignal::new(String::new());
    let email_signal = RwSignal::new(String::new());
    let navigate = use_navigate();
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let username = username_signal.get();
        let password = password_signal.get();
        let email_value = email_signal.get();
        let email = if email_value.is_empty() {
            None
        } else {
            Some(email_value)
        };
        register_action.dispatch(RegisterUser {
            username,
            password,
            email,
        });
    };

    // Effect to handle navigation after successful registration
    let has_refetched = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(Ok(_)) = register_action.value().get() {
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
                navigate("/dashboard", Default::default());
            }
        }
    });

    view! {
        <GuestLayout>
            <div>
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">"Create Account"</h2>

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
                        <InputLabel for_input="email">"Email (Optional)"</InputLabel>
                        <TextInput
                            input_type="email"
                            placeholder="you@example.com"
                            class="block mt-1 w-full"
                            required=false
                            value=email_signal
                        />
                        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">"Optional - for password recovery in the future"</p>
                    </div>

                    <div>
                        <InputLabel for_input="password">"Password"</InputLabel>
                        <TextInput
                            input_type="password"
                            placeholder="Minimum 8 characters"
                            class="block mt-1 w-full"
                            required=true
                            value=password_signal
                        />
                        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">"Must be at least 8 characters long"</p>
                    </div>

                    <div class="flex items-center justify-end">
                        <PrimaryButton
                            button_type="submit"
                            disabled=register_action.pending()
                        >
                            {move || if register_action.pending().get() { "Creating account..." } else { "Register" }}
                        </PrimaryButton>
                    </div>

                    {move || {
                        register_action.value().get().map(|result| {
                            match result {
                                Ok(_) => {
                                    view! {
                                        <div class="rounded-md bg-green-50 dark:bg-green-900/30 p-4">
                                            <p class="text-sm text-green-700 dark:text-green-300">"Registration successful! Redirecting..."</p>
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
                        "Already have an account? "
                        <a href="/login" class="font-medium text-indigo-600 hover:text-indigo-500 dark:text-indigo-400 dark:hover:text-indigo-300">
                            "Login here"
                        </a>
                    </p>
                </div>
            </div>
        </GuestLayout>
    }
}
