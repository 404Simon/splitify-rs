//! Group show page - split into modular components
//!
//! This module contains the main GroupsShow page component and its
//! sub-sections. Each section is separated into its own file for better
//! maintainability.

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::{
    components::{AppLayout, Navigation},
    features::{
        auth::{use_logout, UserSession},
        groups::handlers::{get_group, get_group_members},
        recurring_debts::handlers::{get_recurring_debts, DeleteRecurringDebt},
        shared_debts::handlers::{get_group_shared_debts, DeleteSharedDebt},
        transactions::handlers::{
            calculate_user_debts, delete_transaction, get_group_transactions,
        },
    },
};

mod balances;
mod members;
mod recurring_debts;
mod shared_debts;
mod shopping_lists;
mod transactions;

use balances::BalancesSection;
use members::MembersSection;
use recurring_debts::RecurringDebtsSection;
use shared_debts::SharedDebtsSection;
use shopping_lists::ShoppingListsSection;
use transactions::TransactionsSection;

/// Group show page - displays group details and members
#[must_use]
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

    let recurring_debts_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_recurring_debts(id).await }
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
    let delete_recurring_debt_action = ServerAction::<DeleteRecurringDebt>::new();
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
        if delete_recurring_debt_action.value().get().is_some() {
            recurring_debts_resource.refetch();
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
                                                                // Header section
                                                                <div class="mb-8 flex justify-between items-center">
                                                                    <div>
                                                                        <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">{group.name.clone()}</h1>
                                                                        <p class="text-gray-600 dark:text-gray-400 mt-1">"Group Details"</p>
                                                                    </div>
                                                    {is_admin.then(|| {
                                                        let gid = group_id.get_untracked();
                                                        view! {
                                                        <div class="flex gap-2">
                                                            <a
                                                                href=format!("/groups/{}/invites", gid)
                                                                class="px-4 py-2 bg-indigo-100 hover:bg-indigo-200 dark:bg-indigo-900/30 dark:hover:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300 rounded-lg font-medium transition-colors"
                                                            >
                                                                "Manage Invites"
                                                            </a>
                                                            <a
                                                                href=format!("/groups/{}/edit", gid)
                                                                class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium transition-colors"
                                                            >
                                                                "Edit Group"
                                                            </a>
                                                        </div>
                                                    }})}
                                                                </div>

                                                                // Component sections
                                                                <BalancesSection balances_resource=balances_resource />
                                                                <MembersSection members_resource=members_resource />
                                                                <ShoppingListsSection group_id=group_id />
                                                                <SharedDebtsSection
                                                                    group_id=group_id
                                                                    shared_debts_resource=shared_debts_resource
                                                                    delete_action=delete_debt_action
                                                                />
                                                                <RecurringDebtsSection
                                                                    group_id=group_id
                                                                    recurring_debts_resource=recurring_debts_resource
                                                                    delete_action=delete_recurring_debt_action
                                                                />
                                                                <TransactionsSection
                                                                    group_id=group_id
                                                                    user_id=user.id
                                                                    transactions_resource=transactions_resource
                                                                    delete_action=delete_transaction_action
                                                                />
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
