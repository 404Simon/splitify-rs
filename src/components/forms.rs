use leptos::prelude::*;

/// Form field component with label
#[must_use]
#[component]
pub fn FormField(
    /// Label text
    label: &'static str,
    /// HTML id for the input
    #[prop(optional)]
    for_id: &'static str,
    /// Optional helper text below the input
    #[prop(optional)]
    helper_text: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div>
            <label
                for=for_id
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
            >
                {label}
            </label>
            {children()}
            {helper_text.map(|text| view! {
                <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">{text}</p>
            })}
        </div>
    }
}

/// Text input component with consistent styling
#[must_use]
#[component]
pub fn FormInput(
    /// Input ID
    #[prop(optional)]
    id: &'static str,
    /// Input type (text, email, password, etc.)
    #[prop(default = "text")]
    input_type: &'static str,
    /// Placeholder text
    #[prop(optional)]
    placeholder: &'static str,
    /// Whether the field is required
    #[prop(default = false)]
    required: bool,
    /// Whether the field is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Signal for the value
    #[prop(into)]
    value: Signal<String>,
    /// Setter for the value
    #[prop(into)]
    on_input: Callback<String>,
) -> impl IntoView {
    view! {
        <input
            type=input_type
            id=id
            required=required
            disabled=disabled
            placeholder=placeholder
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white disabled:bg-gray-100 dark:disabled:bg-gray-800 disabled:cursor-not-allowed disabled:text-gray-600 dark:disabled:text-gray-400"
            value=value.get_untracked()
            on:input=move |ev| on_input.run(event_target_value(&ev))
        />
    }
}

/// Number input component with consistent styling
#[must_use]
#[component]
pub fn FormNumberInput(
    /// Input ID
    #[prop(optional)]
    id: &'static str,
    /// Placeholder text
    #[prop(optional)]
    placeholder: &'static str,
    /// Minimum value
    #[prop(optional)]
    min: &'static str,
    /// Step value
    #[prop(default = "0.01")]
    step: &'static str,
    /// Whether the field is required
    #[prop(default = false)]
    required: bool,
    /// Whether the field is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Signal for the value
    #[prop(into)]
    value: Signal<String>,
    /// Setter for the value
    #[prop(into)]
    on_input: Callback<String>,
) -> impl IntoView {
    view! {
        <input
            type="number"
            id=id
            required=required
            disabled=disabled
            step=step
            min=min
            placeholder=placeholder
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white disabled:bg-gray-100 dark:disabled:bg-gray-800 disabled:cursor-not-allowed disabled:text-gray-600 dark:disabled:text-gray-400"
            value=value.get_untracked()
            on:input=move |ev| on_input.run(event_target_value(&ev))
        />
    }
}

/// Date input component with consistent styling
#[must_use]
#[component]
pub fn FormDateInput(
    /// Input ID
    #[prop(optional)]
    id: &'static str,
    /// Whether the field is required
    #[prop(default = false)]
    required: bool,
    /// Whether the field is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Signal for the value
    #[prop(into)]
    value: Signal<String>,
    /// Setter for the value
    #[prop(into)]
    on_input: Callback<String>,
) -> impl IntoView {
    view! {
        <input
            type="date"
            id=id
            required=required
            disabled=disabled
            class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white disabled:bg-gray-100 dark:disabled:bg-gray-800 disabled:cursor-not-allowed disabled:text-gray-600 dark:disabled:text-gray-400"
            value=value.get_untracked()
            on:input=move |ev| on_input.run(event_target_value(&ev))
        />
    }
}

/// Select dropdown component with consistent styling
#[must_use]
#[component]
pub fn FormSelect(
    /// Input ID
    #[prop(optional)]
    id: &'static str,
    /// Whether the field is required
    #[prop(default = false)]
    required: bool,
    /// Whether the field is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Signal for the value
    #[prop(into)]
    value: Signal<String>,
    /// Setter for the value
    #[prop(into)]
    on_change: Callback<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <select
            id=id
            required=required
            disabled=disabled
            class="w-full pl-4 pr-10 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:text-white disabled:bg-gray-100 dark:disabled:bg-gray-800 disabled:cursor-not-allowed appearance-none bg-[length:1.5em] bg-[right_0.5rem_center] bg-no-repeat"
            style="background-image: url('data:image/svg+xml,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 fill=%27none%27 viewBox=%270 0 20 20%27%3E%3Cpath stroke=%27%236b7280%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27 stroke-width=%271.5%27 d=%27M6 8l4 4 4-4%27/%3E%3C/svg%3E')"
            prop:value=move || value.get()
            on:change=move |ev| on_change.run(event_target_value(&ev))
        >
            {children()}
        </select>
    }
}

/// Checkbox list component for member selection
#[must_use]
#[component]
pub fn FormCheckboxList<T, IV>(
    /// The items to display
    #[prop(into)]
    items: Signal<Vec<T>>,
    /// Signal for selected item IDs
    #[prop(into)]
    selected: Signal<Vec<i64>>,
    /// Setter for selected items
    #[prop(into)]
    on_change: Callback<Vec<i64>>,
    /// Function to get the ID from an item
    item_id: fn(&T) -> i64,
    /// Function to render each item
    item_view: fn(T, bool, Callback<(i64, bool)>) -> IV,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    view! {
        <div class="space-y-2 max-h-64 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
            {move || {
                let items_vec = items.get();
                let selected_ids = selected.get();
                items_vec.into_iter().map(|item| {
                    let id = item_id(&item);
                    let is_checked = selected_ids.contains(&id);
                    let on_change_clone = on_change;
                    let checkbox_callback = Callback::new(move |(member_id, checked): (i64, bool)| {
                        let mut current = selected.get();
                        if checked {
                            if !current.contains(&member_id) {
                                current.push(member_id);
                            }
                        } else {
                            current.retain(|&id| id != member_id);
                        }
                        on_change_clone.run(current);
                    });
                    item_view(item, is_checked, checkbox_callback)
                }).collect_view()
            }}
        </div>
    }
}

/// Simple checkbox list item for member selection (most common use case)
#[must_use]
#[component]
pub fn MemberCheckboxItem(
    /// Member ID
    member_id: i64,
    /// Member username
    username: String,
    /// Whether this checkbox is checked
    is_checked: bool,
    /// Callback when checkbox state changes
    #[prop(into)]
    on_change: Callback<bool>,
) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <input
                type="checkbox"
                id=format!("member-{}", member_id)
                checked=is_checked
                class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 dark:border-gray-700 rounded bg-white dark:bg-gray-700"
                on:change=move |ev| {
                    let checked = event_target_checked(&ev);
                    on_change.run(checked);
                }
            />
            <label
                for=format!("member-{}", member_id)
                class="ml-2 text-gray-700 dark:text-gray-300"
            >
                {username}
            </label>
        </div>
    }
}

/// Error alert component
#[must_use]
#[component]
pub fn ErrorAlert(
    /// Error message to display
    #[prop(into)]
    message: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || message.get().map(|msg| view! {
            <div class="rounded-md bg-red-50 dark:bg-red-900/30 p-4">
                <p class="text-sm text-red-700 dark:text-red-300">{msg}</p>
            </div>
        })}
    }
}

/// Success alert component
#[must_use]
#[component]
pub fn SuccessAlert(
    /// Success message to display
    message: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-md bg-green-50 dark:bg-green-900/30 p-4">
            <p class="text-sm text-green-700 dark:text-green-300">{message}</p>
        </div>
    }
}

/// Info alert component
#[must_use]
#[component]
pub fn InfoAlert(
    /// Info message to display
    message: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-md bg-indigo-50 dark:bg-indigo-900/30 p-4">
            <p class="text-sm text-indigo-700 dark:text-indigo-300">{message}</p>
        </div>
    }
}

/// Primary submit button with loading state
#[must_use]
#[component]
pub fn SubmitButton(
    /// Button text when not loading
    text: &'static str,
    /// Button text when loading
    #[prop(optional)]
    loading_text: &'static str,
    /// Whether the button is in loading state
    #[prop(into)]
    loading: Signal<bool>,
) -> impl IntoView {
    let loading_text = if loading_text.is_empty() {
        "Loading..."
    } else {
        loading_text
    };

    view! {
        <button
            type="submit"
            disabled=move || loading.get()
            class="flex-1 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors"
        >
            {move || if loading.get() { loading_text } else { text }}
        </button>
    }
}

/// Cancel button/link component
#[must_use]
#[component]
pub fn CancelButton(
    /// URL to navigate to when clicked
    href: String,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg font-medium transition-colors text-center"
        >
            "Cancel"
        </a>
    }
}

/// Form action buttons container (submit + cancel)
#[must_use]
#[component]
pub fn FormActions(
    /// Submit button text
    submit_text: &'static str,
    /// Submit button loading text
    #[prop(optional)]
    loading_text: &'static str,
    /// Whether the submit button is in loading state
    #[prop(into)]
    loading: Signal<bool>,
    /// Cancel button URL
    cancel_href: String,
) -> impl IntoView {
    view! {
        <div class="flex gap-3">
            <SubmitButton text=submit_text loading_text=loading_text loading=loading />
            <CancelButton href=cancel_href />
        </div>
    }
}

/// Page header component
#[must_use]
#[component]
pub fn PageHeader(
    /// Page title
    title: String,
    /// Optional subtitle/description
    #[prop(optional)]
    subtitle: Option<String>,
) -> impl IntoView {
    view! {
        <div class="mb-8">
            <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-white">
                {title}
            </h1>
            {subtitle.map(|text| view! {
                <p class="text-gray-600 dark:text-gray-400 mt-1">{text}</p>
            })}
        </div>
    }
}

/// Form card container
#[must_use]
#[component]
pub fn FormCard(children: Children) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            {children()}
        </div>
    }
}

/// Loading spinner component
#[must_use]
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex justify-center items-center min-h-screen bg-gray-100 dark:bg-gray-900">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
        </div>
    }
}
