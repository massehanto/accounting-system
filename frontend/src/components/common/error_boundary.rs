use leptos::*;

#[derive(Clone, Debug)]
pub enum AppError {
    NetworkError(String),
    AuthenticationError(String),
    ValidationError(String),
    BusinessLogicError(String),
    UnexpectedError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NetworkError(msg) => write!(f, "Kesalahan jaringan: {}", msg),
            AppError::AuthenticationError(msg) => write!(f, "Kesalahan autentikasi: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Kesalahan validasi: {}", msg),
            AppError::BusinessLogicError(msg) => write!(f, "Kesalahan bisnis: {}", msg),
            AppError::UnexpectedError(msg) => write!(f, "Kesalahan tidak terduga: {}", msg),
        }
    }
}

#[component]
pub fn ErrorBoundary(
    #[prop(optional)] fallback: Option<Box<dyn Fn(AppError) -> View>>,
    children: Children,
) -> impl IntoView {
    let error = create_rw_signal(None::<AppError>);

    let default_fallback = |error: AppError| {
        view! {
            <div class="bg-red-50 border border-red-200 rounded-md p-4">
                <div class="flex">
                    <div class="flex-shrink-0">
                        <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                        </svg>
                    </div>
                    <div class="ml-3">
                        <h3 class="text-sm font-medium text-red-800">
                            "An error occurred"
                        </h3>
                        <div class="mt-2 text-sm text-red-700">
                            {format!("{}", error)}
                        </div>
                    </div>
                </div>
            </div>
        }
    };

    view! {
        <Show
            when=move || error.get().is_some()
            fallback=move || children()
        >
            {move || {
                if let Some(err) = error.get() {
                    if let Some(ref custom_fallback) = fallback {
                        custom_fallback(err)
                    } else {
                        default_fallback(err).into_view()
                    }
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </Show>
    }
}

#[component]
pub fn ErrorAlert(
    error: ReadSignal<Option<AppError>>,
    #[prop(optional)] on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <Show when=move || error.get().is_some()>
            <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
                <div class="flex justify-between">
                    <div class="flex">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <div class="ml-3">
                            <div class="text-sm text-red-700">
                                {move || error.get().map(|e| format!("{}", e)).unwrap_or_default()}
                            </div>
                        </div>
                    </div>
                    <Show when=move || on_dismiss.is_some()>
                        <div class="ml-auto pl-3">
                            <button
                                class="text-red-400 hover:text-red-600"
                                on:click=move |_| {
                                    if let Some(handler) = on_dismiss {
                                        handler.call(());
                                    }
                                }
                            >
                                <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}