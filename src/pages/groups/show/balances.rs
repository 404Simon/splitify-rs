use leptos::prelude::*;

use crate::features::transactions::models::{NetType, RelationshipType, UserBalance};

/// Balance overview section component
#[must_use]
#[component]
pub fn BalancesSection(
    balances_resource: LocalResource<Result<Vec<UserBalance>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mb-6">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">"Balance Overview"</h2>
            <Suspense fallback=move || view! { <div>"Loading balances..."</div> }>
                {move || {
                    match balances_resource.get() {
                        Some(Ok(balances)) if balances.is_empty() => view! {
                            <p class="text-gray-500 dark:text-gray-400 text-center py-4">"No debt information available"</p>
                        }.into_any(),
                        Some(Ok(balances)) => view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                {balances.into_iter().map(|balance| {
                                    view! {
                                        <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg border border-gray-100 dark:border-gray-600">
                                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                                                {balance.username.clone()}
                                            </h3>
                                            <div class="space-y-2 mb-4">
                                                {balance.relationships.into_iter().map(|rel| {
                                                    match rel.relationship_type {
                                                        RelationshipType::Owes => view! {
                                                            <div class="text-sm text-red-600 dark:text-red-400">
                                                                "Owes " {rel.other_username} " " <span class="font-semibold">"€" {rel.amount}</span>
                                                            </div>
                                                        },
                                                        RelationshipType::Owed => view! {
                                                            <div class="text-sm text-green-600 dark:text-green-400">
                                                                "Is owed by " {rel.other_username} " " <span class="font-semibold">"€" {rel.amount}</span>
                                                            </div>
                                                        }
                                                    }
                                                }).collect_view()}
                                            </div>
                                            <div class="pt-4 border-t border-gray-200 dark:border-gray-600 space-y-1">
                                                <div class="text-xs text-gray-600 dark:text-gray-400">
                                                    "Total Owed: " <span class="font-medium text-green-600 dark:text-green-400">"€" {balance.total_owed}</span>
                                                </div>
                                                <div class="text-xs text-gray-600 dark:text-gray-400">
                                                    "Total Owing: " <span class="font-medium text-red-600 dark:text-red-400">"€" {balance.total_owing}</span>
                                                </div>
                                                {match balance.net_type {
                                                    NetType::Positive => view! {
                                                        <div class="text-sm font-semibold text-green-600 dark:text-green-400">
                                                            "Net: +€" {balance.net_amount}
                                                        </div>
                                                    }.into_any(),
                                                    NetType::Negative => view! {
                                                        <div class="text-sm font-semibold text-red-600 dark:text-red-400">
                                                            "Net: -€" {balance.net_amount}
                                                        </div>
                                                    }.into_any(),
                                                    NetType::Neutral => view! {
                                                        <div class="text-sm font-semibold text-gray-600 dark:text-gray-400">
                                                            "Net: €0.00"
                                                        </div>
                                                    }.into_any()
                                                }}
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
