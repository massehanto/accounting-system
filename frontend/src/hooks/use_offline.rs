use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug)]
pub struct OfflineState {
    pub is_online: bool,
    pub sync_pending: bool,
    pub last_sync: Option<String>,
}

pub fn use_offline() -> ReadSignal<OfflineState> {
    let state = create_rw_signal(OfflineState {
        is_online: true,
        sync_pending: false,
        last_sync: None,
    });

    // Monitor online/offline status
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            let is_online = navigator.on_line();
            
            state.update(|s| s.is_online = is_online);
            
            // Setup event listeners
            let state_clone = state;
            let online_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
                state_clone.update(|s| s.is_online = true);
            }) as Box<dyn FnMut(_)>);

            let state_clone2 = state;
            let offline_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
                state_clone2.update(|s| s.is_online = false);
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback("online", online_callback.as_ref().unchecked_ref());
            let _ = window.add_event_listener_with_callback("offline", offline_callback.as_ref().unchecked_ref());

            online_callback.forget();
            offline_callback.forget();
        }
    });

    state.into()
}