// frontend/src/components/forms.rs
use leptos::*;
use leptos::ev::SubmitEvent;
use leptos::html::Select;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[component]
pub fn ValidatedInput(
    label: String,
    input_type: String,
    value: ReadSignal<String>,
    on_input: Callback<String>,
    #[prop(optional)] validator: Option<Box<dyn Fn(&str) -> Option<String>>>,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] required: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
) -> impl IntoView {
    let (error, set_error) = create_signal(Option::<String>::None);
    let (is_touched, set_is_touched) = create_signal(false);

    let validate = move |value: &str| {
        if let Some(ref validator) = validator {
            validator(value)
        } else {
            None
        }
    };

    let on_input_handler = move |ev: ev::Event| {
        let target = ev.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
        let new_value = target.value();
        
        // Validate on input if field has been touched
        if is_touched.get() {
            if let Some(err) = validate(&new_value) {
                set_error.set(Some(err));
            } else {
                set_error.set(None);
            }
        }
        
        on_input.call(new_value);
    };

    let on_blur = move |_| {
        set_is_touched.set(true);
        if let Some(err) = validate(&value.get()) {
            set_error.set(Some(err));
        } else {
            set_error.set(None);
        }
    };

    let has_error = move || error.get().is_some();

    view! {
        <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-2">
                {label.clone()}
                {move || if required.unwrap_or(false) { 
                    view! { <span class="text-red-500">"*"</span> } 
                } else { 
                    view! { <span></span> } 
                }}
            </label>
            <input
                type=input_type
                class=move || format!("form-input {}", 
                    if has_error() { "border-red-500 focus:border-red-500 focus:ring-red-500" } else { "" }
                )
                placeholder=placeholder.unwrap_or_default()
                required=required.unwrap_or(false)
                disabled=disabled.unwrap_or(false)
                prop:value=move || value.get()
                on:input=on_input_handler
                on:blur=on_blur
            />
            <Show when=move || has_error()>
                <p class="text-red-500 text-sm mt-1">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}

#[component]
pub fn ValidatedSelectField(
    label: String,
    value: ReadSignal<String>,
    on_change: Callback<String>,
    options: Vec<(String, String)>, // (value, display_text)
    #[prop(optional)] validator: Option<Box<dyn Fn(&str) -> Option<String>>>,
    #[prop(optional)] required: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
) -> impl IntoView {
    let (error, set_error) = create_signal(Option::<String>::None);
    let (is_touched, set_is_touched) = create_signal(false);

    let validate = move |value: &str| {
        if let Some(ref validator) = validator {
            validator(value)
        } else {
            None
        }
    };

    let on_change_handler = move |ev: ev::Event| {
        let target = ev.target().unwrap().dyn_into::<HtmlSelectElement>().unwrap();
        let new_value = target.value();
        
        if is_touched.get() {
            if let Some(err) = validate(&new_value) {
                set_error.set(Some(err));
            } else {
                set_error.set(None);
            }
        }
        
        on_change.call(new_value);
    };

    let on_blur = move |_| {
        set_is_touched.set(true);
        if let Some(err) = validate(&value.get()) {
            set_error.set(Some(err));
        } else {
            set_error.set(None);
        }
    };

    let has_error = move || error.get().is_some();

    view! {
        <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-2">
                {label.clone()}
                {move || if required.unwrap_or(false) { 
                    view! { <span class="text-red-500">"*"</span> } 
                } else { 
                    view! { <span></span> } 
                }}
            </label>
            <select
                class=move || format!("form-select {}", 
                    if has_error() { "border-red-500 focus:border-red-500 focus:ring-red-500" } else { "" }
                )
                required=required.unwrap_or(false)
                disabled=disabled.unwrap_or(false)
                prop:value=move || value.get()
                on:change=on_change_handler
                on:blur=on_blur
            >
                <option value="">"Select an option"</option>
                {options.into_iter().map(|(val, text)| {
                    view! {
                        <option value=val>{text}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
            <Show when=move || has_error()>
                <p class="text-red-500 text-sm mt-1">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}

pub mod validators {
    pub fn required(value: &str) -> Option<String> {
        if value.trim().is_empty() {
            Some("This field is required".to_string())
        } else {
            None
        }
    }

    pub fn email(value: &str) -> Option<String> {
        if value.trim().is_empty() {
            return None;
        }
        
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if email_regex.is_match(value) {
            None
        } else {
            Some("Please enter a valid email address".to_string())
        }
    }

    pub fn npwp(value: &str) -> Option<String> {
        if value.trim().is_empty() {
            return None;
        }
        
        let npwp_regex = regex::Regex::new(r"^\d{2}\.\d{3}\.\d{3}\.\d{1}-\d{3}\.\d{3}$").unwrap();
        if npwp_regex.is_match(value) {
            None
        } else {
            Some("Please enter a valid NPWP format (XX.XXX.XXX.X-XXX.XXX)".to_string())
        }
    }
}