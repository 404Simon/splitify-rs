use crate::features::auth::get_user;
use crate::pages::{
    GroupsCreate, GroupsEdit, GroupsIndex, GroupsInvites, GroupsShow, HomePage, InviteAccept,
    LoginPage, RegisterPage,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path, StaticSegment,
};

/// Shell function for SSR HTML template
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
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
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Fetch user session on app load
    let user_resource = LocalResource::new(|| async move { get_user().await });

    // Provide user context globally
    provide_context(user_resource);

    view! {
        // Meta tags for better SEO and appearance
        <Meta name="description" content="Rustify Splitify - Split expenses with friends, the Rust way. Fast, secure, and reliable expense tracking."/>
        <Meta name="theme-color" content="#4F46E5"/>

        // Google Fonts - Figtree font family
        <Link rel="preconnect" href="https://fonts.bunny.net"/>
        <Link href="https://fonts.bunny.net/css?family=figtree:400,500,600&display=swap" rel="stylesheet"/>

        // Stylesheet injection
        <Stylesheet id="leptos" href="/pkg/rustify-app.css"/>

        // Document title
        <Title text="Rustify Splitify - Split Expenses with Friends"/>

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
                    <Route path=path!("/invite/:uuid") view=InviteAccept/>
                </Routes>
            </main>
        </Router>
    }
}
