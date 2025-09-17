// frontend/src/components/lazy_loading.rs
use leptos::*;

#[component]
pub fn LazyRoute<T, F>(
    condition: Signal<bool>,
    fallback: F,
    children: Children,
) -> impl IntoView 
where 
    T: IntoView + 'static,
    F: Fn() -> T + 'static,
{
    view! {
        <Suspense fallback=fallback>
            <Show when=condition>
                {children()}
            </Show>
        </Suspense>
    }
}

#[component] 
pub fn PageLoader() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center min-h-screen">
            <div class="text-center">
                <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mb-4"></div>
                <p class="text-gray-600">"Memuat halaman..."</p>
            </div>
        </div>
    }
}

#[component]
pub fn LazyImage(
    src: String,
    alt: String,
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] loading: Option<String>,
) -> impl IntoView {
    let (loaded, set_loaded) = create_signal(false);
    let (error, set_error) = create_signal(false);

    view! {
        <div class={format!("relative {}", class.unwrap_or_default())}>
            <Show when=move || !loaded.get() && !error.get()>
                <div class="animate-pulse bg-gray-200 w-full h-full min-h-[100px] rounded"></div>
            </Show>
            
            <img
                src=src
                alt=alt
                class={move || format!("transition-opacity duration-300 {}", 
                    if loaded.get() { "opacity-100" } else { "opacity-0" })}
                loading=loading.unwrap_or_else(|| "lazy".to_string())
                on:load=move |_| set_loaded.set(true)
                on:error=move |_| set_error.set(true)
            />
            
            <Show when=move || error.get()>
                <div class="flex items-center justify-center bg-gray-100 w-full h-full min-h-[100px] rounded">
                    <span class="text-gray-400 text-sm">"Failed to load image"</span>
                </div>
            </Show>
        </div>
    }
}