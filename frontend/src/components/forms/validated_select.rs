use leptos::*;
use web_sys::HtmlSelectElement;

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