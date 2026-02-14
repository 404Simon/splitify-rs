use leptos::prelude::*;

use crate::features::groups::models::GroupMemberInfo;

/// Group members section component
#[must_use]
#[component]
pub fn MembersSection(
    members_resource: LocalResource<Result<Vec<GroupMemberInfo>, ServerFnError>>,
) -> impl IntoView {
    view! {
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
    }
}
