use leptos::prelude::*;

use crate::features::recurring_debts::{
    handlers::DeleteRecurringDebt, models::RecurringDebtWithDetails,
};

/// Recurring debts section component
#[must_use]
#[component]
pub fn RecurringDebtsSection(
    group_id: Memo<i64>,
    recurring_debts_resource: LocalResource<Result<Vec<RecurringDebtWithDetails>, ServerFnError>>,
    delete_action: ServerAction<DeleteRecurringDebt>,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mt-6">
            <div class="flex justify-between items-center mb-4">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Recurring Debts"</h2>
                <a
                    href=format!("/groups/{}/recurring-debts/create", group_id.get())
                    class="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-medium transition-colors inline-flex items-center"
                >
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    "Add Recurring Debt"
                </a>
            </div>
            <Suspense fallback=move || view! { <div>"Loading recurring debts..."</div> }>
                {move || {
                    match recurring_debts_resource.get() {
                        Some(Ok(debts)) if debts.is_empty() => view! {
                            <div class="text-center py-12">
                                <div class="w-16 h-16 mx-auto mb-4 bg-purple-100 dark:bg-purple-900/30 rounded-full flex items-center justify-center">
                                    <svg class="w-8 h-8 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                                    </svg>
                                </div>
                                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">"No recurring debts yet"</h3>
                                <p class="text-gray-500 dark:text-gray-400 mb-6 text-sm">"Set up automatic debt generation for recurring expenses."</p>
                            </div>
                        }.into_any(),
                        Some(Ok(debts)) => view! {
                            <div class="space-y-4">
                                {debts.into_iter().map(|debt| {
                                    let recurring_id = debt.id;
                                    let gid = group_id.get();
                                    view! {
                                        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 border border-gray-100 dark:border-gray-600">
                                            <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3">
                                                <div class="flex-1 min-w-0">
                                                    <div class="flex items-center gap-2 mb-1">
                                                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate">{debt.name.clone()}</h3>
                                                        <span class={format!(
                                                            "px-2 py-0.5 rounded text-xs font-medium {}",
                                                            if debt.is_active {
                                                                "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                                                            } else {
                                                                "bg-gray-200 text-gray-700 dark:bg-gray-600 dark:text-gray-300"
                                                            }
                                                        )}>
                                                            {if debt.is_active { "Active" } else { "Paused" }}
                                                        </span>
                                                    </div>
                                                    <p class="text-2xl font-bold text-purple-600 dark:text-purple-400">
                                                        "€" {format!("{:.2}", debt.amount)}
                                                    </p>
                                                    <p class="text-sm text-gray-600 dark:text-gray-400 capitalize">
                                                        {debt.frequency.to_string()} " • Next: " {debt.next_generation_date.to_string()}
                                                    </p>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400">
                                                        "Created by " {debt.creator_username.clone()}
                                                    </p>
                                                </div>
                                                <div class="flex flex-wrap gap-2">
                                                    <a
                                                        href=format!("/groups/{}/recurring-debts/{}", gid, recurring_id)
                                                        class="px-3 py-1.5 bg-purple-100 hover:bg-purple-200 dark:bg-purple-900/30 dark:hover:bg-purple-900/50 text-purple-700 dark:text-purple-300 rounded-lg text-sm font-medium transition-colors inline-flex items-center"
                                                    >
                                                        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
                                                        </svg>
                                                        "View"
                                                    </a>
                                                    {debt.is_creator.then(|| view! {
                                                        <button
                                                            on:click=move |_| {
                                                                if window().confirm_with_message("Are you sure you want to delete this recurring debt? Generated debts will remain, but no new ones will be created.").unwrap_or(false) {
                                                                    delete_action.dispatch(DeleteRecurringDebt { recurring_debt_id: recurring_id });
                                                                }
                                                            }
                                                            class="px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition-colors inline-flex items-center"
                                                        >
                                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                                            </svg>
                                                            "Delete"
                                                        </button>
                                                    })}
                                                </div>
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
