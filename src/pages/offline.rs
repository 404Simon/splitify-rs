use leptos::prelude::*;

/// Offline fallback page displayed when network is unavailable
#[component]
pub fn OfflinePage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900 px-4">
            <div class="text-center max-w-md">
                <div class="mb-8">
                    <svg
                        class="mx-auto h-24 w-24 text-gray-400 dark:text-gray-600"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-2.83m-1.414 5.658a9 9 0 01-2.167-9.238m7.824 2.167a1 1 0 111.414 1.414m-1.414-1.414L3 3m8.293 8.293l1.414 1.414"
                        />
                    </svg>
                </div>

                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-4">
                    "You're Offline"
                </h1>

                <p class="text-lg text-gray-600 dark:text-gray-400 mb-6">
                    "It looks like you've lost your internet connection. Please check your network and try again."
                </p>

                <div class="space-y-4">
                    <button
                        on:click=move |_| {
                            window().location().reload().unwrap_or_default();
                        }
                        class="w-full inline-flex items-center justify-center px-6 py-3 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg shadow-lg transition-all duration-200"
                    >
                        <svg
                            class="w-5 h-5 mr-2"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                            />
                        </svg>
                        "Retry"
                    </button>

                    <a
                        href="/"
                        class="block w-full px-6 py-3 bg-gray-200 hover:bg-gray-300 dark:bg-gray-800 dark:hover:bg-gray-700 text-gray-900 dark:text-white font-semibold rounded-lg shadow transition-all duration-200"
                    >
                        "Go to Home"
                    </a>
                </div>

                <div class="mt-8 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                    <p class="text-sm text-blue-800 dark:text-blue-300">
                        <span class="font-semibold">"💡 Tip:"</span>
                        " Some cached pages may still be available. Try navigating to pages you've visited before."
                    </p>
                </div>
            </div>
        </div>
    }
}
