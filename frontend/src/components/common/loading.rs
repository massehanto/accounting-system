// frontend/src/components/common/loading.rs
use leptos::*;

#[component]
pub fn LoadingSpinner(
    #[prop(optional)] size: Option<String>, // "sm", "md", "lg"
    #[prop(optional)] message: Option<String>,
) -> impl IntoView {
    let size = size.unwrap_or_else(|| "md".to_string());
    
    let spinner_classes = match size.as_str() {
        "sm" => "h-4 w-4",
        "lg" => "h-12 w-12",
        _ => "h-8 w-8", // md
    };

    view! {
        <div class="flex flex-col justify-center items-center">
            <div class={format!("animate-spin rounded-full border-b-2 border-primary-600 {}", spinner_classes)}></div>
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

#[component]
pub fn SkeletonLoader(
    #[prop(optional)] lines: Option<usize>,
    #[prop(optional)] height: Option<String>,
) -> impl IntoView {
    let lines = lines.unwrap_or(3);
    let height = height.unwrap_or_else(|| "h-4".to_string());

    view! {
        <div class="animate-pulse space-y-3">
            {(0..lines).map(|i| {
                let width = match i {
                    0 => "w-3/4",
                    n if n == lines - 1 => "w-1/2",
                    _ => "w-full",
                };
                view! {
                    <div class={format!("bg-gray-300 rounded {}", height)}>
                        <div class={format!("bg-gray-300 rounded {}", width)}></div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}