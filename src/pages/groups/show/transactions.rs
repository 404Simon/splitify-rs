use leptos::prelude::*;

use crate::features::transactions::models::TransactionWithDetails;

/// Transactions section component
#[must_use]
#[component]
pub fn TransactionsSection(
    group_id: Memo<i64>,
    user_id: i64,
    transactions_resource: LocalResource<Result<Vec<TransactionWithDetails>, ServerFnError>>,
    delete_action: Action<(i64, i64), Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mt-6">
            <div class="flex justify-between items-center mb-4">
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Transactions"</h2>
                <a
                    href=move || format!("/groups/{}/transactions/create", group_id.get())
                    class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-medium transition-colors inline-flex items-center"
                >
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                    "Add Transaction"
                </a>
            </div>
            <Suspense fallback=move || view! { <div>"Loading transactions..."</div> }>
                {move || {
                    match transactions_resource.get() {
                        Some(Ok(transactions)) if transactions.is_empty() => view! {
                            <div class="text-center py-12">
                                <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center">
                                    <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z"/>
                                    </svg>
                                </div>
                                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">"No transactions yet"</h3>
                                <p class="text-gray-500 dark:text-gray-400 mb-6 text-sm">"Record payments between group members to settle debts."</p>
                            </div>
                        }.into_any(),
                        Some(Ok(transactions)) => view! {
                            <div class="space-y-4">
                                {transactions.into_iter().map(|transaction| {
                                    let trans_id = transaction.id;
                                    let gid = group_id.get_untracked();
                                    let is_payer = transaction.payer_id == user_id;
                                    view! {
                                        <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 border border-gray-100 dark:border-gray-600">
                                            <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3">
                                                <div class="flex-1 min-w-0">
                                                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                                        {transaction.description.clone().unwrap_or_else(|| "Payment".to_string())}
                                                    </h3>
                                                    <p class="text-2xl font-bold text-emerald-600 dark:text-emerald-400">
                                                        "€" {format!("{:.2}", transaction.amount.parse::<f64>().unwrap_or(0.0))}
                                                    </p>
                                                    <p class="text-sm text-gray-600 dark:text-gray-400">
                                                        {transaction.payer_username.clone()} " → " {transaction.recipient_username.clone()} " • "
                                                        {transaction.created_at.date().to_string()}
                                                    </p>
                                                </div>
                                                {is_payer.then(|| view! {
                                                    <div class="flex flex-wrap gap-2">
                                                        <a
                                                            href=format!("/groups/{}/transactions/{}/edit", gid, trans_id)
                                                            class="px-3 py-1.5 bg-gray-200 hover:bg-gray-300 dark:bg-gray-600 dark:hover:bg-gray-500 text-gray-900 dark:text-white rounded-lg text-sm font-medium transition-colors inline-flex items-center"
                                                        >
                                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                                                            </svg>
                                                            "Edit"
                                                        </a>
                                                        <button
                                                            on:click=move |_| {
                                                                if window().confirm_with_message("Are you sure you want to delete this transaction?").unwrap_or(false) {
                                                                    delete_action.dispatch((gid, trans_id));
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
                                                })}
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
