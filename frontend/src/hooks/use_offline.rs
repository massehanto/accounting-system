// frontend/src/hooks/use_offline.rs
use leptos::*;
use web_sys::{window, Navigator};

#[derive(Clone, Debug)]
pub struct OfflineState {
    pub is_online: bool,
    pub sync_pending: bool,
    pub last_sync: Option<String>,
}

pub fn use_offline() -> (ReadSignal<OfflineState>, WriteSignal<OfflineState>) {
    let (state, set_state) = create_signal(OfflineState {
        is_online: true,
        sync_pending: false,
        last_sync: None,
    });

    // Monitor online/offline status
    create_effect(move |_| {
        if let Some(window) = window() {
            let navigator = window.navigator();
            let is_online = navigator.on_line();
            
            set_state.update(|s| s.is_online = is_online);
            
            // Setup event listeners for online/offline events
            setup_connectivity_listeners(set_state);
        }
    });

    (state, set_state)
}

fn setup_connectivity_listeners(set_state: WriteSignal<OfflineState>) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    if let Some(window) = window() {
        let online_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            set_state.update(|s| s.is_online = true);
            web_sys::console::log_1(&"🟢 Connection restored".into());
        }) as Box<dyn FnMut(_)>);

        let offline_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            set_state.update(|s| s.is_online = false);
            web_sys::console::log_1(&"🔴 Connection lost".into());
        }) as Box<dyn FnMut(_)>);

        let _ = window.add_event_listener_with_callback("online", online_callback.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("offline", offline_callback.as_ref().unchecked_ref());

        online_callback.forget();
        offline_callback.forget();
    }
}