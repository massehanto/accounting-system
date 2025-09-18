use leptos::*;
use web_sys::HtmlInputElement;

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