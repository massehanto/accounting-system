use leptos::*;

#[derive(Clone, Copy, Debug)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Success,
    Warning,
}

#[derive(Clone, Copy, Debug)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

#[component]
pub fn Button(
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] loading: Option<bool>,
    #[prop(optional)] full_width: Option<bool>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let size = size.unwrap_or(ButtonSize::Medium);
    let disabled = disabled.unwrap_or(false);
    let loading = loading.unwrap_or(false);
    let full_width = full_width.unwrap_or(false);

    let base_classes = "inline-flex justify-center items-center font-medium rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";
    
    let variant_classes = match variant {
        ButtonVariant::Primary => "text-white bg-primary-600 hover:bg-primary-700 focus:ring-primary-500 active:transform active:scale-95",
        ButtonVariant::Secondary => "text-gray-700 bg-white border border-gray-300 hover:bg-gray-50 hover:border-gray-400 focus:ring-primary-500 active:transform active:scale-95",
        ButtonVariant::Danger => "text-white bg-red-600 hover:bg-red-700 focus:ring-red-500 active:transform active:scale-95",
        ButtonVariant::Success => "text-white bg-green-600 hover:bg-green-700 focus:ring-green-500 active:transform active:scale-95",
        ButtonVariant::Warning => "text-white bg-yellow-600 hover:bg-yellow-700 focus:ring-yellow-500 active:transform active:scale-95",
    };
    
    let size_classes = match size {
        ButtonSize::Small => "px-3 py-1.5 text-sm",
        ButtonSize::Medium => "px-4 py-2.5 text-sm",
        ButtonSize::Large => "px-6 py-3 text-base",
    };
    
    let width_classes = if full_width { "w-full" } else { "" };
    
    let classes = format!("{} {} {} {}", base_classes, variant_classes, size_classes, width_classes);

    view! {
        <button
            class=classes
            disabled=move || disabled || loading
            on:click=move |_| {
                if let Some(handler) = on_click {
                    handler.call(());
                }
            }
        >
            <Show when=move || loading>
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
            </Show>
            {children()}
        </button>
    }
}