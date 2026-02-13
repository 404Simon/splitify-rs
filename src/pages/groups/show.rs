use crate::components::{AppLayout, Navigation};
use crate::features::auth::{use_logout, UserSession};
use crate::features::groups::handlers::{get_group, get_group_members};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::use_params_map;

/// Group show page - displays group details and members
#[component]
pub fn GroupsShow() -> impl IntoView {
    let user_resource =
        expect_context::<LocalResource<Result<Option<UserSession>, ServerFnError>>>();
    let navigate = use_navigate();
    let on_logout = use_logout();
    let params = use_params_map();

    let group_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
            .unwrap_or(0)
    });

    let group_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group(id).await }
    });

    let members_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_members(id).await }
    });

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
                            <Navigation username=user.username.clone() on_logout=on_logout />
                            <AppLayout>
                                <div class="py-6">
                                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <Suspense fallback=move || view! { <div>"Loading group..."</div> }>
                                            {move || {
                                                match group_resource.get() {
                                                    Some(Ok(group)) => {
                                                        let is_admin = group.created_by == user.id;
                                                        view! {
                                                            <div>
                                                    <div class="mb-8 flex justify-between items-center">
                                                        <div>
                                                            <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">{group.name.clone()}</h1>
                                                            <p class="text-gray-600 dark:text-gray-400 mt-1">"Group Details"</p>
                                                        </div>
                                                        {is_admin.then(|| view! {
                                                            <div class="flex gap-2">
                                                                <a
                                                                    href=format!("/groups/{}/invites", group_id.get())
                                                                    class="px-4 py-2 bg-indigo-100 hover:bg-indigo-200 dark:bg-indigo-900/30 dark:hover:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300 rounded-lg font-medium transition-colors"
                                                                >
                                                                    "Manage Invites"
                                                                </a>
                                                                <a
                                                                    href=format!("/groups/{}/edit", group_id.get())
                                                                    class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium transition-colors"
                                                                >
                                                                    "Edit Group"
                                                                </a>
                                                            </div>
                                                        })}
                                                    </div>

                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
                                                                    <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Group Members"</h2>
                                                                    <Suspense fallback=move || view! { <div>"Loading members..."</div> }>
                                                                        {move || {
                                                                            match members_resource.get() {
                                                                                Some(Ok(members)) => view! {
                                                                                    <div class="space-y-2">
                                                                                        {members.into_iter().map(|member| view! {
                                                                                            <div class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700">
                                                                                                <div class="flex items-center">
                                                                                                    <div class="w-10 h-10 rounded-full bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center mr-3">
                                                                                                        <span class="text-indigo-600 dark:text-indigo-400 font-semibold">
                                                                                                            {member.username.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                                                                                        </span>
                                                                                                    </div>
                                                                                                    <span class="text-gray-900 dark:text-white font-medium">{member.username}</span>
                                                                                                </div>
                                                                                                {member.is_creator.then(|| view! {
                                                                                                    <span class="px-2 py-1 bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 text-xs font-medium rounded">"Admin"</span>
                                                                                                })}
                                                                                            </div>
                                                                                        }).collect_view()}
                                                                                    </div>
                                                                                }.into_any(),
                                                                                Some(Err(e)) => view! {
                                                                                    <div class="text-red-600 dark:text-red-400">"Error: " {e.to_string()}</div>
                                                                                }.into_any(),
                                                                                None => view! { <div>"Loading..."</div> }.into_any()
                                                                            }
                                                                        }}
                                                                    </Suspense>
                                                                </div>

                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-8 text-center">
                                                                    <p class="text-gray-500 dark:text-gray-400">"More features coming soon!"</p>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    },
                                                    Some(Err(e)) => view! {
                                                        <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                                                            <p class="text-sm text-red-700 dark:text-red-300">"Error: " {e.to_string()}</p>
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
