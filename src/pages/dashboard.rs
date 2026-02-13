use crate::components::{AppLayout, Navigation};
use crate::features::auth::{LogoutUser, UserSession};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Protected dashboard page component
#[component]
pub fn Dashboard() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let logout_action = ServerAction::<LogoutUser>::new();
    let navigate = use_navigate();

    let on_logout = Callback::new(move |_: ()| {
        logout_action.dispatch(LogoutUser {});
    });

    // Effect to handle navigation and resource refresh after logout
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(())) = logout_action.value().get() {
            // Refetch user resource to clear the cached user
            user_resource.refetch();
            // Navigate to home page
            navigate_clone("/", Default::default());
        }
    });

    // Effect to redirect to login if not authenticated
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate("/login", Default::default());
        }
    });

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
                            <Navigation
                                username=user.username.clone()
                                on_logout=on_logout
                            />

                            <AppLayout>
                                <div class="py-6">
                                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <div class="mb-8">
                                            <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">"Dashboard"</h1>
                                            <p class="text-gray-600 dark:text-gray-400 mt-1">"Welcome back, " {user.username.clone()} "!"</p>
                                        </div>

                                        // Empty state - no groups yet
                                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-8 sm:p-12 text-center">
                                            <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center">
                                                <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                        d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                                </svg>
                                            </div>
                                            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">"No groups yet"</h3>
                                            <p class="text-gray-500 dark:text-gray-400 mb-6 text-sm sm:text-base">
                                                "Create or join a group to start splitting expenses with friends."
                                            </p>
                                            <button
                                                class="inline-flex items-center px-6 py-3 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg shadow-md hover:shadow-lg transition-all duration-200"
                                            >
                                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                                                </svg>
                                                "Create Your First Group"
                                            </button>
                                        </div>

                                        // Stats cards (placeholder for future features)
                                        <div class="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
                                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                <div class="flex items-center justify-between">
                                                    <div>
                                                        <p class="text-sm text-gray-600 dark:text-gray-400 font-medium">"Total Groups"</p>
                                                        <p class="text-3xl font-bold text-gray-900 dark:text-white mt-2">"0"</p>
                                                    </div>
                                                    <div class="w-12 h-12 bg-indigo-100 dark:bg-indigo-900/30 rounded-lg flex items-center justify-center">
                                                        <svg class="w-6 h-6 text-indigo-600 dark:text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                                        </svg>
                                                    </div>
                                                </div>
                                            </div>

                                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                <div class="flex items-center justify-between">
                                                    <div>
                                                        <p class="text-sm text-gray-600 dark:text-gray-400 font-medium">"Total Expenses"</p>
                                                        <p class="text-3xl font-bold text-gray-900 dark:text-white mt-2">"$0.00"</p>
                                                    </div>
                                                    <div class="w-12 h-12 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
                                                        <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                                        </svg>
                                                    </div>
                                                </div>
                                            </div>

                                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                <div class="flex items-center justify-between">
                                                    <div>
                                                        <p class="text-sm text-gray-600 dark:text-gray-400 font-medium">"Your Balance"</p>
                                                        <p class="text-3xl font-bold text-gray-900 dark:text-white mt-2">"$0.00"</p>
                                                    </div>
                                                    <div class="w-12 h-12 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center">
                                                        <svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
                                                        </svg>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        {move || {
                                            logout_action.value().get().and_then(|result| {
                                                match result {
                                                    Ok(_) => None,
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
                            </AppLayout>
                        </div>
                    }.into_any(),
                    Some(Ok(None)) => view! {
                        <div class="flex justify-center items-center min-h-screen bg-gray-100 dark:bg-gray-900">
                            <p class="text-gray-600 dark:text-gray-400">"Redirecting to login..."</p>
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
