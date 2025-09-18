use leptos::*;

#[derive(Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[component]
pub fn Select(
    #[prop(into)] label: String,
    value: RwSignal<String>,
    options: Vec<SelectOption>,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] required: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] error: Option<RwSignal<Option<String>>>,
) -> impl IntoView {
    let required = required.unwrap_or(false);
    let disabled = disabled.unwrap_or(false);
    
    let has_error = move || {
        error.map(|e| e.get().is_some()).unwrap_or(false)
    };

    view! {
        <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-2">
                {label.clone()}
                {move || if required { 
                    view! { <span class="text-red-500">"*"</span> } 
                } else { 
                    view! { <span></span> } 
                }}
            </label>
            <select
                class=move || format!(
                    "block w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 sm:text-sm {}",
                    if has_error() { 
                        "border-red-300 focus:border-red-500 focus:ring-red-500" 
                    } else { 
                        "border-gray-300 focus:border-blue-500" 
                    }
                )
                required=required
                disabled=disabled
                prop:value=move || value.get()
                on:change=move |ev| {
                    value.set(event_target_value(&ev));
                }
            >
                <Show when=move || placeholder.is_some()>
                    <option value="">{placeholder.clone().unwrap_or_default()}</option>
                </Show>
                {options.into_iter().map(|option| {
                    view! {
                        <option value=option.value>{option.label}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
            <Show when=move || has_error()>
                {move || {
                    if let Some(error_signal) = error {
                        view! {
                            <p class="text-red-500 text-sm mt-1">
                                {move || error_signal.get().unwrap_or_default()}
                            </p>
                        }
                    } else {
                        view! { <span></span> }
                    }
                }}
            </Show>
        </div>
    }
}