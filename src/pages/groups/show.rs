use crate::components::{AppLayout, Navigation};
use crate::features::auth::{use_logout, UserSession};
use crate::features::groups::handlers::{get_group, get_group_members};
use crate::features::shared_debts::handlers::{
    get_group_shared_debts, get_shared_debt_shares, DeleteSharedDebt,
};
use crate::features::transactions::handlers::{
    calculate_user_debts, delete_transaction, get_group_transactions,
};
use crate::features::transactions::models::{NetType, RelationshipType};
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

    let shared_debts_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_shared_debts(id).await }
    });

    let balances_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { calculate_user_debts(id).await }
    });

    let transactions_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_transactions(id).await }
    });

    let delete_debt_action = ServerAction::<DeleteSharedDebt>::new();
    let delete_transaction_action = Action::new(move |(gid, tid): &(i64, i64)| {
        let gid = *gid;
        let tid = *tid;
        async move { delete_transaction(gid, tid).await }
    });

    // Refetch resources after deletions
    Effect::new(move |_| {
        if delete_debt_action.value().get().is_some() {
            shared_debts_resource.refetch();
            balances_resource.refetch(); // Recalculate balances
        }
    });

    Effect::new(move |_| {
        if delete_transaction_action.value().get().is_some() {
            transactions_resource.refetch();
            balances_resource.refetch(); // Recalculate balances
        }
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

                                                                    // Balance Overview Section
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

                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
                                                                    <div class="flex justify-between items-center mb-4">
                                                                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Shared Debts"</h2>
                                                                        <a
                                                                            href=format!("/groups/{}/debts/create", group_id.get())
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
                                                                                                        {debt.is_creator.then(|| view! {
                                                                                                            <div class="flex flex-wrap gap-2">
                                                                                                                <a
                                                                                                                    href=format!("/groups/{}/debts/{}/edit", group_id.get(), debt.id)
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
                                                                                                                            delete_debt_action.dispatch(DeleteSharedDebt { debt_id: debt.id });
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

                                                                // Transactions Section
                                                                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6 mt-6">
                                                                    <div class="flex justify-between items-center mb-4">
                                                                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Transactions"</h2>
                                                                        <a
                                                                            href=format!("/groups/{}/transactions/create", group_id.get())
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
                                                                                            let gid = group_id.get();
                                                                                            let is_payer = transaction.payer_id == user.id;
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
                                                                                                                            delete_transaction_action.dispatch((gid, trans_id));
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
