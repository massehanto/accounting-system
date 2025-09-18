use leptos::*;
use crate::hooks::use_offline::OfflineState;

#[component]
pub fn OfflineIndicator() -> impl IntoView {
    let offline_state = use_context::<ReadSignal<OfflineState>>();

    view! {
        <Show when=move || {
            if let Some(state) = offline_state {
                !state.get().is_online
            } else {
                false
            }
        }>
            <div class="fixed top-0 left-0 right-0 bg-red-600 text-white text-center py-2 text-sm z-50">
                "⚠️ No internet connection - working offline"
            </div>
        </Show>
    }
}