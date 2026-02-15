use leptos::prelude::*;

use crate::features::shared_debts::{
    handlers::{get_shared_debt_shares, DeleteSharedDebt},
    models::SharedDebtWithDetails,
};

/// Shared debts section component
#[must_use]
#[component]
pub fn SharedDebtsSection(
    group_id: Memo<i64>,
    shared_debts_resource: LocalResource<Result<Vec<SharedDebtWithDetails>, ServerFnError>>,
    delete_action: ServerAction<DeleteSharedDebt>,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            <div class="flex justify-between items-center mb-4">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Shared Debts"</h2>
                <a
                    href=move || format!("/groups/{}/debts/create", group_id.get())
                    class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors inline-flex items-center"
                >
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    "Add Debt"
                </a>
            </div>
            <Suspense fallback=move || view! { <div>"Loading debts..."</div> }>
                {move || {
                    match shared_debts_resource.get() {
                        Some(Ok(debts)) if debts.is_empty() => view! {
                            <div class="text-center py-12">
                                <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center">
                                    <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                    </svg>
                                </div>
                                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">"No shared debts yet"</h3>
                                <p class="text-gray-500 dark:text-gray-400 mb-6 text-sm">"Start tracking shared expenses with your group members."</p>
                            </div>
                        }.into_any(),
                        Some(Ok(debts)) => view! {
                            <div class="space-y-4">
                                {debts.into_iter().map(|debt| {
                                    let debt_id = debt.id;
                                    let shares_resource = LocalResource::new(move || async move { get_shared_debt_shares(debt_id).await });

                                    view! {
                                        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 border border-gray-100 dark:border-gray-600">
                                            <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3">
                                                <div class="flex-1 min-w-0">
                                                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate">{debt.name.clone()}</h3>
                                                    <p class="text-2xl font-bold text-red-600 dark:text-red-400">
                                                        "€" {format!("{:.2}", debt.amount)}
                                                    </p>
                                                    <p class="text-sm text-gray-600 dark:text-gray-400">
                                                        "Created by " {debt.creator_username.clone()} " • "
                                                        {debt.created_at.date().to_string()}
                                                    </p>
                                                </div>
                                                {debt.is_creator.then(|| {
                                                    let gid = group_id.get_untracked();
                                                    let debt_id = debt.id;
                                                    view! {
                                                    <div class="flex flex-wrap gap-2">
                                                        <a
                                                            href=format!("/groups/{}/debts/{}/edit", gid, debt_id)
                                                            class="px-3 py-1.5 bg-gray-200 hover:bg-gray-300 dark:bg-gray-600 dark:hover:bg-gray-500 text-gray-900 dark:text-white rounded-lg text-sm font-medium transition-colors inline-flex items-center"
                                                        >
                                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                                                            </svg>
                                                            "Edit"
                                                        </a>
                                                        <button
                                                            on:click=move |_| {
                                                                if window().confirm_with_message("Are you sure you want to delete this debt?").unwrap_or(false) {
                                                                    delete_action.dispatch(DeleteSharedDebt { debt_id: debt.id });
                                                                }
                                                            }
                                                            class="px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition-colors inline-flex items-center"
                                                        >
                                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                                            </svg>
                                                            "Delete"
                                                        </button>
                                                    </div>
                                                }})}
                                            </div>
                                            <div class="mt-3">
                                                <Suspense fallback=move || view! { <div class="text-xs text-gray-500">"Loading shares..."</div> }>
                                                    {move || {
                                                        match shares_resource.get() {
                                                            Some(Ok(shares)) => view! {
                                                                <div>
                                                                    <p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
                                                                        "Split between " {shares.len().to_string()} " member(s):"
                                                                    </p>
                                                                    <div class="flex flex-wrap gap-2">
                                                                        {shares.into_iter().map(|share| view! {
                                                                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400">
                                                                                {share.username} ": €" {format!("{:.2}", share.share_amount)}
                                                                            </span>
                                                                        }).collect_view()}
                                                                    </div>
                                                                </div>
                                                            }.into_any(),
                                                            Some(Err(_)) => view! {
                                                                <p class="text-xs text-red-500">"Error loading shares"</p>
                                                            }.into_any(),
                                                            None => view! { <div class="text-xs text-gray-500">"..."</div> }.into_any()
                                                        }
                                                    }}
                                                </Suspense>
                                            </div>
                                        </div>
                                    }
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
