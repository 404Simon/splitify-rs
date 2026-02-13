use crate::features::auth::UserSession;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Home page component
#[component]
pub fn HomePage() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let navigate = use_navigate();

    // Redirect logged-in users to /groups
    Effect::new(move |_| {
        if let Some(Ok(Some(_))) = user_resource.get() {
            navigate("/groups", Default::default());
        }
    });

    view! {
        <div class="min-h-screen bg-gradient-to-br from-indigo-50 via-white to-purple-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-950">
            <div class="container mx-auto px-4 py-16">
                <Suspense fallback=move || view! {
                    <div class="flex justify-center items-center min-h-screen">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                    </div>
                }>
                    {move || {
                        match user_resource.get() {
                            Some(Ok(Some(user))) => view! {
                                // Authenticated user view
                                <div class="max-w-4xl mx-auto text-center">
                                    <div class="mb-8">
                                        <div class="inline-flex items-center justify-center w-24 h-24 bg-indigo-600 dark:bg-indigo-500 rounded-full mb-6 shadow-lg">
                                            <span class="text-4xl font-bold text-white">"S"</span>
                                        </div>
                                        <h1 class="text-5xl md:text-6xl font-extrabold text-gray-900 dark:text-white mb-4">
                                            "Welcome back, " <span class="text-indigo-600 dark:text-indigo-400">{user.username.clone()}</span> "!"
                                        </h1>
                                        <p class="text-xl text-gray-600 dark:text-gray-300 mb-8">
                                            "Continue managing your shared expenses"
                                        </p>
                                    </div>

                                    <div class="flex flex-col sm:flex-row gap-4 justify-center items-center">
                                        <a
                                            href="/groups"
                                            class="inline-flex items-center px-8 py-4 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:-translate-y-0.5"
                                        >
                                            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                                            </svg>
                                            "Go to Groups"
                                        </a>
                                    </div>
                                </div>
                            }.into_any(),
                            _ => view! {
                                // Guest view
                                <div class="max-w-5xl mx-auto">
                                    <div class="text-center mb-16">
                                        <div class="inline-flex items-center justify-center w-24 h-24 bg-indigo-600 dark:bg-indigo-500 rounded-full mb-8 shadow-xl">
                                            <span class="text-4xl font-bold text-white">"S"</span>
                                        </div>
                                        <h1 class="text-5xl md:text-7xl font-extrabold text-gray-900 dark:text-white mb-6">
                                            "Rustify " <span class="text-indigo-600 dark:text-indigo-400">"Splitify"</span>
                                        </h1>
                                        <p class="text-xl md:text-2xl text-gray-600 dark:text-gray-300 mb-4">
                                            "Split expenses with friends, the Rust way"
                                        </p>
                                        <p class="text-lg text-gray-500 dark:text-gray-400">
                                            "Fast, secure, and reliable expense tracking built with modern technology"
                                        </p>
                                    </div>

                                    <div class="flex flex-col sm:flex-row gap-4 justify-center items-center mb-20">
                                        <a
                                            href="/login"
                                            class="inline-flex items-center px-8 py-4 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg shadow-lg hover:shadow-xl transition-all duration-200 transform hover:-translate-y-0.5"
                                        >
                                            "Login"
                                        </a>
                                        <a
                                            href="/register"
                                            class="inline-flex items-center px-8 py-4 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 text-indigo-600 dark:text-indigo-400 font-semibold rounded-lg shadow-lg hover:shadow-xl border-2 border-indigo-600 dark:border-indigo-400 transition-all duration-200 transform hover:-translate-y-0.5"
                                        >
                                            "Get Started"
                                        </a>
                                    </div>

                                    // Features section
                                    <div class="grid md:grid-cols-3 gap-8 mt-16">
                                        <div class="bg-white dark:bg-gray-800 p-8 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 hover:shadow-xl transition-shadow duration-200">
                                            <div class="w-12 h-12 bg-indigo-100 dark:bg-indigo-900/30 rounded-lg flex items-center justify-center mb-4">
                                                <svg class="w-6 h-6 text-indigo-600 dark:text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                                </svg>
                                            </div>
                                            <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">"Group Expenses"</h3>
                                            <p class="text-gray-600 dark:text-gray-400">"Create groups and track shared expenses with friends, roommates, or travel companions."</p>
                                        </div>

                                        <div class="bg-white dark:bg-gray-800 p-8 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 hover:shadow-xl transition-shadow duration-200">
                                            <div class="w-12 h-12 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center mb-4">
                                                <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                                                </svg>
                                            </div>
                                            <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">"Easy Calculations"</h3>
                                            <p class="text-gray-600 dark:text-gray-400">"Automatically calculate who owes what with precise decimal math powered by Rust."</p>
                                        </div>

                                        <div class="bg-white dark:bg-gray-800 p-8 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 hover:shadow-xl transition-shadow duration-200">
                                            <div class="w-12 h-12 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center mb-4">
                                                <svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                                </svg>
                                            </div>
                                            <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">"Secure & Fast"</h3>
                                            <p class="text-gray-600 dark:text-gray-400">"Built with Rust for blazing performance and rock-solid security you can trust."</p>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
