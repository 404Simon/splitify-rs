use leptos::prelude::*;

/// Splitify pie chart icon component
#[must_use]
#[component]
pub fn SplitifyIcon(#[prop(optional)] class: &'static str) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" class=class>
            <circle cx="100" cy="100" r="80" fill="#6b7280"/>
            <path d="M 100 100 L 180 100 A 80 80 0 0 1 156.56 156.56 Z" fill="#4b5563" stroke="#1f2937" stroke-width="1"/>
            <path d="M 100 100 L 156.56 156.56 A 80 80 0 0 1 100 180 Z" fill="#374151" stroke="#1f2937" stroke-width="1"/>
            <path d="M 100 100 L 100 180 A 80 80 0 0 1 43.44 156.56 Z" fill="#4b5563" stroke="#1f2937" stroke-width="1"/>
            <path d="M 100 100 L 43.44 156.56 A 80 80 0 0 1 20 100 Z" fill="#374151" stroke="#1f2937" stroke-width="1"/>
            <path d="M 100 100 L 20 100 A 80 80 0 0 1 43.44 43.44 Z" fill="#4b5563" stroke="#1f2937" stroke-width="1"/>
            <path d="M 100 100 L 43.44 43.44 A 80 80 0 0 1 100 20 Z" fill="#374151" stroke="#1f2937" stroke-width="1"/>
            <path d="M 100 100 L 100 20 A 80 80 0 0 1 156.56 43.44 Z" fill="#4b5563" stroke="#1f2937" stroke-width="1"/>
            <path d="M 100 100 L 156.56 43.44 A 80 80 0 0 1 180 100 Z" fill="#374151" stroke="#1f2937" stroke-width="1"/>
        </svg>
    }
}

/// Primary button component matching Laravel's styling
#[must_use]
#[component]
pub fn PrimaryButton(
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] button_type: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type=button_type
            disabled=move || disabled.get()
            class="inline-flex items-center px-4 py-2 bg-gray-800 dark:bg-gray-200 border border-transparent rounded-md font-semibold text-xs text-white dark:text-gray-800 uppercase tracking-widest hover:bg-gray-700 dark:hover:bg-white focus:bg-gray-700 dark:focus:bg-white active:bg-gray-900 dark:active:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 disabled:opacity-50 transition ease-in-out duration-150"
        >
            {children()}
        </button>
    }
}

/// Secondary button component
#[must_use]
#[component]
pub fn SecondaryButton(
    #[prop(optional)] disabled: bool,
    #[prop(optional)] button_type: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type=button_type
            disabled=disabled
            class="inline-flex items-center px-4 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-500 rounded-md font-semibold text-xs text-gray-700 dark:text-gray-300 uppercase tracking-widest shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 disabled:opacity-50 transition ease-in-out duration-150"
        >
            {children()}
        </button>
    }
}

/// Text input component matching Laravel's styling
#[must_use]
#[component]
pub fn TextInput(
    #[prop(optional)] input_type: &'static str,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] required: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] class: &'static str,
    value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <input
            type=input_type
            placeholder=placeholder
            required=required
            disabled=disabled
            class=format!("px-3 py-2 border border-gray-300 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-300 focus:border-indigo-500 dark:focus:border-indigo-600 focus:ring-indigo-500 dark:focus:ring-indigo-600 rounded-md shadow-sm {}", class)
            on:input=move |ev| value.set(event_target_value(&ev))
            prop:value=move || value.get()
        />
    }
}

/// Input label component
#[must_use]
#[component]
pub fn InputLabel(#[prop(optional)] for_input: &'static str, children: Children) -> impl IntoView {
    view! {
        <label
            for=for_input
            class="block font-medium text-sm text-gray-700 dark:text-gray-300"
        >
            {children()}
        </label>
    }
}

/// Guest layout wrapper (for login/register pages)
#[must_use]
#[component]
pub fn GuestLayout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col sm:justify-center items-center pt-6 sm:pt-0 bg-gray-100 dark:bg-gray-900">
            <div>
                <a href="/">
                    <div class="w-20 h-20">
                        <SplitifyIcon class="w-full h-full"/>
                    </div>
                </a>
            </div>

            <div class="w-full sm:max-w-md mt-6 px-6 py-4 bg-white dark:bg-gray-800 shadow-md overflow-hidden sm:rounded-lg">
                {children()}
            </div>
        </div>
    }
}

/// Navigation component for authenticated pages
#[must_use]
#[component]
pub fn Navigation(username: String, #[prop(into)] on_logout: Callback<()>) -> impl IntoView {
    let (open, set_open) = signal(false);

    view! {
        <nav class="bg-white dark:bg-gray-800 border-b border-gray-100 dark:border-gray-700">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between h-16">
                    <div class="flex">
                        // Logo
                        <div class="shrink-0 flex items-center">
                            <a href="/groups" class="flex items-center">
                                <div class="w-9 h-9">
                                    <SplitifyIcon class="w-full h-full"/>
                                </div>
                            </a>
                        </div>

                        // Navigation Links
                        <div class="hidden space-x-8 sm:-my-px sm:ms-10 sm:flex">
                            <a
                                href="/groups"
                                class="inline-flex items-center px-1 pt-1 border-b-2 border-indigo-400 dark:border-indigo-600 text-sm font-medium leading-5 text-gray-900 dark:text-gray-100 focus:outline-none focus:border-indigo-700 transition duration-150 ease-in-out"
                            >
                                "Groups"
                            </a>
                        </div>
                    </div>

                    // Settings Dropdown (Desktop)
                    <div class="hidden sm:flex sm:items-center sm:ms-6">
                        <div class="relative">
                            <button
                                on:click=move |_| set_open.set(!open.get())
                                class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-gray-500 dark:text-gray-400 bg-white dark:bg-gray-800 hover:text-gray-700 dark:hover:text-gray-300 focus:outline-none transition ease-in-out duration-150"
                            >
                                <div>{username.clone()}</div>
                                <div class="ms-1">
                                    <svg class="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                                    </svg>
                                </div>
                            </button>

                            <Show when=move || open.get()>
                                <div class="absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5">
                                    <button
                                    on:click=move |_| {
                                        on_logout.run(());
                                        set_open.set(false);
                                    }
                                        class="block w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-900"
                                    >
                                        "Log Out"
                                    </button>
                                </div>
                            </Show>
                        </div>
                    </div>

                    // Hamburger (Mobile)
                    <div class="-me-2 flex items-center sm:hidden">
                        <button
                            on:click=move |_| set_open.set(!open.get())
                            class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 dark:text-gray-500 hover:text-gray-500 dark:hover:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-900 focus:outline-none focus:bg-gray-100 dark:focus:bg-gray-900 focus:text-gray-500 dark:focus:text-gray-400 transition duration-150 ease-in-out"
                        >
                            <svg class="h-6 w-6" stroke="currentColor" fill="none" viewBox="0 0 24 24">
                                <Show
                                    when=move || open.get()
                                    fallback=move || view! {
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                                    }
                                >
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </Show>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            // Responsive Navigation Menu
            <Show when=move || open.get()>
                <div class="sm:hidden">
                    <div class="pt-2 pb-3 space-y-1">
                        <a
                            href="/groups"
                            class="block ps-3 pe-4 py-2 border-l-4 border-indigo-400 dark:border-indigo-600 text-base font-medium text-indigo-700 dark:text-indigo-300 bg-indigo-50 dark:bg-indigo-900/50 focus:outline-none focus:text-indigo-800 dark:focus:text-indigo-200 focus:bg-indigo-100 dark:focus:bg-indigo-900 focus:border-indigo-700 dark:focus:border-indigo-300 transition duration-150 ease-in-out"
                        >
                            "Groups"
                        </a>
                    </div>

                    <div class="pt-4 pb-1 border-t border-gray-200 dark:border-gray-600">
                        <div class="px-4">
                            <div class="font-medium text-base text-gray-800 dark:text-gray-200">{username.clone()}</div>
                        </div>

                        <div class="mt-3 space-y-1">
                            <button
                                on:click=move |_| {
                                    on_logout.run(());
                                    set_open.set(false);
                                }
                                class="block w-full text-left ps-3 pe-4 py-2 border-l-4 border-transparent text-base font-medium text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 hover:border-gray-300 dark:hover:border-gray-600 focus:outline-none focus:text-gray-800 dark:focus:text-gray-200 focus:bg-gray-50 dark:focus:bg-gray-700 focus:border-gray-300 dark:focus:border-gray-600 transition duration-150 ease-in-out"
                            >
                                "Log Out"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </nav>
    }
}

/// App layout wrapper (for authenticated pages)
#[must_use]
#[component]
pub fn AppLayout(
    #[prop(optional)] header_title: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
            {header_title.map(|title| {
                view! {
                    <header class="bg-white dark:bg-gray-800 shadow">
                        <div class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
                            <h2 class="font-semibold text-xl text-gray-800 dark:text-gray-200 leading-tight">
                                {title}
                            </h2>
                        </div>
                    </header>
                }
            })}

            <main>
                {children()}
            </main>
        </div>
    }
}
