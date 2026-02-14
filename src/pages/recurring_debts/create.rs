use crate::components::{
    AppLayout, ErrorAlert, FormActions, FormCard, FormDateInput, FormField, FormInput,
    FormNumberInput, FormSelect, LoadingSpinner, MemberCheckboxItem, Navigation, PageHeader,
};
use crate::features::auth::{use_logout, UserSession};
use crate::features::groups::handlers::{get_group, get_group_members};
use crate::features::recurring_debts::handlers::CreateRecurringDebt;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

/// Create recurring debt page
#[component]
pub fn RecurringDebtsCreate() -> impl IntoView {
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

    let create_action = ServerAction::<CreateRecurringDebt>::new();
    let (name, set_name) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (frequency, set_frequency) = signal("monthly".to_string());
    let (start_date, set_start_date) = signal(String::new());
    let (end_date, set_end_date) = signal(String::new());
    let (selected_members, set_selected_members) = signal(Vec::<i64>::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Effect to redirect if not authenticated
    let navigate_clone = navigate.clone();
    Effect::new(move |_| {
        if let Some(Ok(None)) = user_resource.get() {
            navigate_clone("/login", Default::default());
        }
    });

    // Effect to initialize selected members (all members by default)
    Effect::new(move |_| {
        if let Some(Ok(members)) = members_resource.get() {
            if selected_members.get().is_empty() {
                set_selected_members.set(members.iter().map(|m| m.id).collect());
            }
        }
    });

    // Effect to handle submission result
    Effect::new(move |_| {
        if let Some(result) = create_action.value().get() {
            match result {
                Ok(_) => {
                    navigate(&format!("/groups/{}", group_id.get()), Default::default());
                }
                Err(e) => {
                    set_error_message.set(Some(e.to_string()));
                }
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(None);

        // Convert empty end_date to None
        let end_date_value = end_date.get();
        let end_date_opt = if end_date_value.is_empty() {
            None
        } else {
            Some(end_date_value)
        };

        create_action.dispatch(CreateRecurringDebt {
            group_id: group_id.get(),
            name: name.get(),
            amount: amount.get(),
            frequency: frequency.get(),
            start_date: start_date.get(),
            end_date: end_date_opt,
            member_ids: selected_members.get(),
        });
    };

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
                                        <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                                            {move || {
                                                match group_resource.get() {
                                                    Some(Ok(group)) => view! {
                                                        <PageHeader title=format!("Add Recurring Debt to {}", group.name) />

                                                        <FormCard>
                                                            <form on:submit=on_submit class="space-y-6">
                                                                <ErrorAlert message=error_message />

                                                                <FormField label="Name" for_id="name">
                                                                    <FormInput
                                                                        id="name"
                                                                        placeholder="e.g., Monthly rent"
                                                                        required=true
                                                                        value=Signal::derive(move || name.get())
                                                                        on_input=Callback::new(move |val| set_name.set(val))
                                                                    />
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

                                                                <FormField label="Frequency" for_id="frequency">
                                                                    <FormSelect
                                                                        id="frequency"
                                                                        required=true
                                                                        value=Signal::derive(move || frequency.get())
                                                                        on_change=Callback::new(move |val| set_frequency.set(val))
                                                                    >
                                                                        <option value="daily">"Daily"</option>
                                                                        <option value="weekly">"Weekly"</option>
                                                                        <option value="monthly" selected>"Monthly"</option>
                                                                        <option value="yearly">"Yearly"</option>
                                                                    </FormSelect>
                                                                </FormField>

                                                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                                                    <FormField label="Start Date" for_id="start_date">
                                                                        <FormDateInput
                                                                            id="start_date"
                                                                            required=true
                                                                            value=Signal::derive(move || start_date.get())
                                                                            on_input=Callback::new(move |val| set_start_date.set(val))
                                                                        />
                                                                    </FormField>

                                                                    <FormField label="End Date (Optional)" for_id="end_date">
                                                                        <FormDateInput
                                                                            id="end_date"
                                                                            value=Signal::derive(move || end_date.get())
                                                                            on_input=Callback::new(move |val| set_end_date.set(val))
                                                                        />
                                                                    </FormField>
                                                                </div>

                                                                <FormField label="Split Between">
                                                                    <Suspense fallback=move || view! { <div>"Loading members..."</div> }>
                                                                        {move || {
                                                                            match members_resource.get() {
                                                                                Some(Ok(members)) => {
                                                                                    let selected = selected_members.get();
                                                                                    view! {
                                                                                        <div class="space-y-2 max-h-64 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
                                                                                            {members.into_iter().map(|member| {
                                                                                                let member_id = member.id;
                                                                                                let is_checked = selected.contains(&member_id);
                                                                                                view! {
                                                                                                    <MemberCheckboxItem
                                                                                                        member_id=member_id
                                                                                                        username=member.username
                                                                                                        is_checked=is_checked
                                                                                                        on_change=Callback::new(move |checked| {
                                                                                                            set_selected_members.update(|members| {
                                                                                                                if checked {
                                                                                                                    if !members.contains(&member_id) {
                                                                                                                        members.push(member_id);
                                                                                                                    }
                                                                                                                } else {
                                                                                                                    members.retain(|&id| id != member_id);
                                                                                                                }
                                                                                                            });
                                                                                                        })
                                                                                                    />
                                                                                                }
                                                                                            }).collect_view()}
                                                                                        </div>
                                                                                    }.into_any()
                                                                                },
                                                                                Some(Err(e)) => view! {
                                                                                    <div class="text-red-600 dark:text-red-400">"Error: " {e.to_string()}</div>
                                                                                }.into_any(),
                                                                                None => view! { <div>"Loading..."</div> }.into_any()
                                                                            }
                                                                        }}
                                                                    </Suspense>
                                                                </FormField>

                                                                <FormActions
                                                                    submit_text="Create Recurring Debt"
                                                                    loading_text="Creating..."
                                                                    loading=Signal::derive(move || create_action.pending().get())
                                                                    cancel_href=format!("/groups/{}", group_id.get())
                                                                />
                                                            </form>
                                                        </FormCard>
                                                    }.into_any(),
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
                    _ => LoadingSpinner().into_any()
                }
            }}
        </Suspense>
    }
}
