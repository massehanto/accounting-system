use leptos::*;

#[component]
pub fn LoadingSpinner(
    #[prop(optional)] size: Option<String>,
    #[prop(optional)] message: Option<String>,
) -> impl IntoView {
    let size = size.unwrap_or_else(|| "md".to_string());
    
    let spinner_classes = match size.as_str() {
        "sm" => "h-4 w-4",
        "lg" => "h-12 w-12",
        _ => "h-8 w-8",
    };

    view! {
        <div class="flex flex-col justify-center items-center">
            <div class=format!("animate-spin rounded-full border-b-2 border-blue-600 {}", spinner_classes)></div>
            <Show when=move || message.is_some()>
                <p class="mt-2 text-sm text-gray-600">{message.clone().unwrap_or_default()}</p>
            </Show>
        </div>
    }
}

#[component]
pub fn LoadingOverlay(
    show: ReadSignal<bool>,
    #[prop(optional)] message: Option<String>,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 bg-white bg-opacity-75 flex items-center justify-center z-50">
                <LoadingSpinner size="lg".to_string() message=message.clone()/>
            </div>
        </Show>
    }
}