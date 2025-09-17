// frontend/src/components/lazy_loading.rs
use leptos::*;

#[component]
pub fn LazyRoute<T>(
    condition: Signal<bool>,
    fallback: impl Fn() -> T + 'static,
    children: Children,
) -> impl IntoView 
where 
    T: IntoView + 'static
{
    view! {
        <Suspense fallback=move || fallback()>
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