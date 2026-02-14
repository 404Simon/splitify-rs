use leptos::{prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
};

use crate::{
    components::{
        AppLayout, ErrorAlert, FormActions, FormCard, FormField, FormInput, FormNumberInput,
        FormSelect, LoadingSpinner, Navigation, PageHeader,
    },
    features::{
        auth::{models::UserSession, use_logout},
        groups::handlers::get_group_members,
        transactions::handlers::create_transaction,
    },
};

#[must_use]
#[component]
pub fn TransactionsCreate() -> impl IntoView {
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

    let members_resource = LocalResource::new(move || {
        let id = group_id.get();
        async move { get_group_members(id).await }
    });

    let (recipient_id, set_recipient_id) = signal(String::from("0"));
    let (amount, set_amount) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let (current_user_id, set_current_user_id) = signal(0i64);
    let (is_submitting, set_is_submitting) = signal(false);

    // Set current_user_id when user loads
    Effect::new(move |_| {
        if let Some(Ok(Some(user))) = user_resource.get() {
            set_current_user_id.set(user.id);
        }
    });

    // Effect to redirect if not authenticated
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate_clone("/login", Default::default());
        }
    });

    // Clone navigate for use in on_submit
    let navigate_for_submit = navigate.clone();

    let on_submit = StoredValue::new(move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(None);
        set_is_submitting.set(true);

        let gid = group_id.get();
        let rid_str = recipient_id.get();
        let amt = amount.get();
        let desc = description.get();
        let nav = navigate_for_submit.clone();

        spawn_local(async move {
            let rid = match rid_str.parse::<i64>() {
                Ok(id) if id != 0 => id,
                _ => {
                    set_error_message.set(Some("Please select a recipient".to_string()));
                    set_is_submitting.set(false);
                    return;
                }
            };

            if amt.is_empty() {
                set_error_message.set(Some("Please enter an amount".to_string()));
                set_is_submitting.set(false);
                return;
            }

            let desc_opt = if desc.is_empty() { None } else { Some(desc) };

            match create_transaction(gid, rid, amt, desc_opt).await {
                Ok(_) => {
                    nav(&format!("/groups/{}", gid), Default::default());
                }
                Err(e) => {
                    set_error_message.set(Some(e.to_string()));
                    set_is_submitting.set(false);
                }
            }
        });
    });

    view! {
        <Suspense fallback=LoadingSpinner>
            {move || {
                match user_resource.get() {
                    Some(Ok(Some(user))) => view! {
                        <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
                            <Navigation username=user.username.clone() on_logout=on_logout />
                            <AppLayout>
                                <div class="py-6">
                                    <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
                                        <A href=format!("/groups/{}", group_id.get()) attr:class="text-indigo-600 dark:text-indigo-400 hover:text-indigo-900 dark:hover:text-indigo-300 text-sm inline-flex items-center mb-3">
                                            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                                            </svg>
                                            "Back to Group"
                                        </A>

                                        <PageHeader title="Add Transaction".to_string() />

                                        <FormCard>
                                            <form on:submit=move |ev| on_submit.with_value(|f| f(ev)) class="space-y-6">
                                                <ErrorAlert message=error_message />

                                                <FormField label="Recipient" for_id="recipient_id">
                                                    <Suspense fallback=move || view! { <div>"Loading members..."</div> }>
                                                        {move || {
                                                            match members_resource.get() {
                                                                Some(Ok(members)) => {
                                                                    let other_members: Vec<_> = members
                                                                        .into_iter()
                                                                        .filter(|m| m.id != current_user_id.get())
                                                                        .collect();
                                                                    view! {
                                                                        <FormSelect
                                                                            id="recipient_id"
                                                                            required=true
                                                                            value=Signal::derive(move || recipient_id.get())
                                                                            on_change=Callback::new(move |val| set_recipient_id.set(val))
                                                                        >
                                                                            <option value="0">"Select Recipient"</option>
                                                                            {other_members
                                                                                .into_iter()
                                                                                .map(|member| {
                                                                                    view! {
                                                                                        <option value=member.id.to_string()>
                                                                                            {member.username}
                                                                                        </option>
                                                                                    }
                                                                                })
                                                                                .collect_view()}
                                                                        </FormSelect>
                                                                    }.into_any()
                                                                }
                                                                Some(Err(e)) => view! {
                                                                    <div class="text-red-600 dark:text-red-400">"Error: " {e.to_string()}</div>
                                                                }.into_any(),
                                                                None => view! { <div>"Loading..."</div> }.into_any()
                                                            }
                                                        }}
                                                    </Suspense>
                                                </FormField>

                                                <FormField label="Amount (€)" for_id="amount">
                                                    <FormNumberInput
                                                        id="amount"
                                                        placeholder="0.00"
                                                        min="0.01"
                                                        step="0.01"
                                                        required=true
                                                        value=Signal::derive(move || amount.get())
                                                        on_input=Callback::new(move |val| set_amount.set(val))
                                                    />
                                                </FormField>

                                                <FormField label="Description (optional)" for_id="description">
                                                    <FormInput
                                                        id="description"
                                                        placeholder="e.g., Dinner payment"
                                                        value=Signal::derive(move || description.get())
                                                        on_input=Callback::new(move |val| set_description.set(val))
                                                    />
                                                </FormField>

                                                <FormActions
                                                    submit_text="Add Transaction"
                                                    loading_text="Adding..."
                                                    loading=Signal::derive(move || is_submitting.get())
                                                    cancel_href=format!("/groups/{}", group_id.get())
                                                />
                                            </form>
                                        </FormCard>
                                    </div>
                                </div>
                            </AppLayout>
                        </div>
                    }.into_any(),
                    _ => LoadingSpinner().into_any()
                }
            }}
        </Suspense>
    }
}
