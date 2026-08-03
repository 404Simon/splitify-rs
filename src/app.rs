use leptos::prelude::*;
use leptos_meta::{Link, Meta, MetaTags, Script, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
    path,
};

use crate::{
    features::auth::get_user,
    pages::{
        GroupMap, GroupsCreate, GroupsEdit, GroupsIndex, GroupsInvites, GroupsShow, HomePage,
        InviteAccept, LoginPage, RecurringDebtsCreate, RecurringDebtsEdit, RecurringDebtsShow,
        RegisterPage, SharedDebtsCreate, SharedDebtsEdit, ShoppingListCreate, ShoppingListEdit,
        ShoppingListShow, TransactionsCreate, TransactionsEdit,
    },
};

/// Shell function for SSR HTML template
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                // Set the theme class before first paint to avoid a flash.
                // Mirrored in src/components/layout.rs (ThemeToggle).
                // An explicit `light` class is kept alongside `dark` so that
                // non-Leptos consumers (e.g. the map glue) can tell an explicit
                // light choice apart from an unset one (which falls back to the
                // OS preference).
                <script>
                    (function () {
                        try {
                            var stored = localStorage.getItem("splitify-theme");
                            var dark = stored
                                ? stored === "dark"
                                : window.matchMedia("(prefers-color-scheme: dark)").matches;
                            document.documentElement.classList.toggle("dark", dark);
                            document.documentElement.classList.toggle("light", !dark);
                        } catch (e) {}
                    })();
                </script>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="font-sans antialiased dark:bg-gray-900">
                <App/>
            </body>
        </html>
    }
}

/// Root application component
#[must_use]
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Fetch user session on app load
    let user_resource = LocalResource::new(|| async move { get_user().await });

    // Provide user context globally
    provide_context(user_resource);

    // Theme management: a reactive `dark` flag consumed by the navbar toggle.
    // The initial value comes from the `<html class="dark">` set by the shell's
    // inline script; toggling it applies the class and persists the choice.
    let dark_mode = RwSignal::new(false);
    provide_context(dark_mode);

    #[cfg(feature = "hydrate")]
    {
        use leptos::prelude::*;

        let initial_dark = document()
            .document_element()
            .map(|element| element.class_list().contains("dark"))
            .unwrap_or(false);
        dark_mode.set(initial_dark);

        Effect::new(move |_| {
            let is_dark = dark_mode.get();
            if let Some(element) = document().document_element() {
                // Keep the explicit `light` class in sync alongside `dark` (see
                // the shell script) so consumers can distinguish an explicit
                // choice from an unset one.
                if is_dark {
                    let _ = element.class_list().add_1("dark");
                    let _ = element.class_list().remove_1("light");
                } else {
                    let _ = element.class_list().remove_1("dark");
                    let _ = element.class_list().add_1("light");
                }
            }
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.set_item("splitify-theme", if is_dark { "dark" } else { "light" });
            }
        });
    }

    view! {
        // Meta tags for better SEO and appearance
        <Meta name="description" content="Splitify - Split expenses with friends, the Rust way. Fast, secure, and reliable expense tracking."/>
        <Meta name="theme-color" content="#4F46E5"/>
        <Meta name="mobile-web-app-capable" content="yes"/>
        <Meta name="apple-mobile-web-app-status-bar-style" content="black-translucent"/>

        // Favicon and app icons
        <Link rel="icon" type_="image/svg+xml" href="/favicon.svg"/>
        <Link rel="icon" sizes="192x192" href="/favicon-192x192.png"/>
        <Link rel="apple-touch-icon" href="/favicon-192x192.png"/>

        // Google Fonts - Figtree font family
        <Link rel="preconnect" href="https://fonts.bunny.net"/>
        <Link href="https://fonts.bunny.net/css?family=figtree:400,500,600&display=swap" rel="stylesheet"/>

        // Stylesheet injection
        <Stylesheet id="leptos" href="/pkg/splitify.css"/>

        // MapLibre glue bundle (built by `pnpm build:map`)
        <Stylesheet id="maplibre" href="/maplibre/map.css"/>
        <Script src="/maplibre/map.mjs" type_="module"/>

        // Document title
        <Title text="Splitify - Split Expenses with Friends"/>

        // Main router and content
        <Router>
            <main>
                <Routes fallback=|| view! {
                    <div class="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
                        <div class="text-center">
                            <h1 class="text-6xl font-bold text-gray-900 dark:text-white mb-4">"404"</h1>
                            <p class="text-xl text-gray-600 dark:text-gray-400 mb-8">"Page not found"</p>
                            <a
                                href="/"
                                class="inline-flex items-center px-6 py-3 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg shadow-lg transition-all duration-200"
                            >
                                "Go Home"
                            </a>
                        </div>
                    </div>
                }.into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("register") view=RegisterPage/>
                    <Route path=StaticSegment("groups") view=GroupsIndex/>
                    <Route path=path!("/groups/create") view=GroupsCreate/>
                    <Route path=path!("/groups/:id") view=GroupsShow/>
                    <Route path=path!("/groups/:id/edit") view=GroupsEdit/>
                    <Route path=path!("/groups/:id/invites") view=GroupsInvites/>
                    <Route path=path!("/groups/:id/map") view=GroupMap/>
                    <Route path=path!("/groups/:id/debts/create") view=SharedDebtsCreate/>
                    <Route path=path!("/groups/:id/debts/:debt_id/edit") view=SharedDebtsEdit/>
                    <Route path=path!("/groups/:group_id/shopping-lists/create") view=ShoppingListCreate/>
                    <Route path=path!("/groups/:group_id/shopping-lists/:list_id") view=ShoppingListShow/>
                    <Route path=path!("/groups/:group_id/shopping-lists/:list_id/edit") view=ShoppingListEdit/>
                    <Route path=path!("/groups/:id/recurring-debts/create") view=RecurringDebtsCreate/>
                    <Route path=path!("/groups/:id/recurring-debts/:recurring_id") view=RecurringDebtsShow/>
                    <Route path=path!("/groups/:id/recurring-debts/:recurring_id/edit") view=RecurringDebtsEdit/>
                    <Route path=path!("/groups/:id/transactions/create") view=TransactionsCreate/>
                    <Route path=path!("/groups/:id/transactions/:transaction_id/edit") view=TransactionsEdit/>
                    <Route path=path!("/invite/:uuid") view=InviteAccept/>
                </Routes>
            </main>
        </Router>
    }
}
