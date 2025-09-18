use leptos::*;

#[component]
pub fn Modal(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] size: Option<String>,
    children: Children,
) -> impl IntoView {
    let size = size.unwrap_or_else(|| "md".to_string());
    
    let modal_classes = match size.as_str() {
        "sm" => "max-w-sm",
        "lg" => "max-w-2xl",
        "xl" => "max-w-4xl",
        _ => "max-w-lg",
    };

    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50 flex items-center justify-center p-4">
                <div class=format!("relative bg-white rounded-lg shadow-xl w-full {} max-h-[90vh] overflow-y-auto", modal_classes)>
                    <Show when=move || title.is_some()>
                        <div class="flex justify-between items-center px-6 py-4 border-b border-gray-200">
                            <h3 class="text-lg font-medium text-gray-900">
                                {title.clone().unwrap_or_default()}
                            </h3>
                            <button 
                                class="text-gray-400 hover:text-gray-600 transition-colors"
                                on:click=move |_| on_close.call(())
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>
                    </Show>
                    <div class="px-6 py-5">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}