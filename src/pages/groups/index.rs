use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::{
    components::{AppLayout, Navigation},
    features::{
        auth::{use_logout, UserSession},
        groups::handlers::get_user_groups,
    },
};

/// Groups index page - lists all user's groups
#[must_use]
#[component]
pub fn GroupsIndex() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let navigate = use_navigate();
    let on_logout = use_logout();

    let groups_resource = LocalResource::new(move || async move { get_user_groups().await });

    // Effect to redirect if not authenticated
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
                                        <div class="mb-8 flex justify-between items-center">
                                            <div>
                                                <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">"My Groups"</h1>
                                                <p class="text-gray-600 dark:text-gray-400 mt-1">"Manage your expense groups"</p>
                                            </div>
                                            <a
                                                href="/groups/create"
                                                class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg shadow-md hover:shadow-lg transition-all duration-200"
                                            >
                                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                                                </svg>
                                                "Create Group"
                                            </a>
                                        </div>

                                        <Suspense fallback=move || view! { <div>"Loading groups..."</div> }>
                                            {move || {
                                                match groups_resource.get() {
                                                    Some(Ok(groups)) => {
                                                        if groups.is_empty() {
                                                            view! {
                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-8 sm:p-12 text-center">
                                                                    <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center">
                                                                        <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                                                d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                                                        </svg>
                                                                    </div>
                                                                    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">"No groups yet"</h3>
                                                                    <p class="text-gray-500 dark:text-gray-400 mb-6">"Create your first group to start splitting expenses."</p>
                                                                    <a
                                                                        href="/groups/create"
                                                                        class="inline-flex items-center px-6 py-3 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg shadow-md hover:shadow-lg transition-all duration-200"
                                                                    >
                                                                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                                                                        </svg>
                                                                        "Create Your First Group"
                                                                    </a>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                                                                    {groups.into_iter().map(|group| view! {
                                                                        <a
                                                                            href=format!("/groups/{}", group.id)
                                                                            class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 hover:shadow-lg transition-all duration-200"
                                                                        >
                                                                            <div class="flex justify-between items-start mb-4">
                                                                                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{group.name}</h3>
                                                                                {group.is_admin.then(|| view! {
                                                                                    <span class="px-2 py-1 bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 text-xs font-medium rounded">"Admin"</span>
                                                                                })}
                                                                            </div>
                                                                            <div class="flex items-center text-sm text-gray-600 dark:text-gray-400">
                                                                                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                                                                                </svg>
                                                                                {format!("{} member{}", group.member_count, if group.member_count == 1 { "" } else { "s" })}
                                                                            </div>
                                                                        </a>
                                                                    }).collect_view()}
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    },
                                                    Some(Err(e)) => view! {
                                                        <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                            <p class="text-sm text-red-700 dark:text-red-300">"Error loading groups: " {e.to_string()}</p>
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
