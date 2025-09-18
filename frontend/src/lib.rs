use leptos::*;

// Core modules
pub mod app;
pub mod components;
pub mod pages;
pub mod api;
pub mod types;
pub mod utils;
pub mod error;
pub mod stores;
pub mod hooks;
pub mod config;

// Re-exports for easier imports
pub use app::*;
pub use error::*;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Enhanced PWA initialization
    initialize_pwa();
    
    mount_to_body(App)
}

fn initialize_pwa() {
    // Service Worker registration with better error handling
    if let Some(window) = web_sys::window() {
        if let Ok(navigator) = window.navigator().service_worker() {
            wasm_bindgen_futures::spawn_local(async move {
                match navigator.register("/sw.js").await {
                    Ok(_) => web_sys::console::log_1(&"✅ Service Worker registered".into()),
                    Err(e) => web_sys::console::error_1(&format!("❌ SW registration failed: {:?}", e).into()),
                }
            });
        }
    }

    // PWA install prompt handling
    setup_pwa_install_prompt();
}

fn setup_pwa_install_prompt() {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    
    if let Some(window) = web_sys::window() {
        let before_install_prompt = Closure::wrap(Box::new(move |event: web_sys::Event| {
            event.prevent_default();
            web_sys::console::log_1(&"💾 PWA install prompt available".into());
        }) as Box<dyn FnMut(_)>);
        
        let _ = window.add_event_listener_with_callback(
            "beforeinstallprompt",
            before_install_prompt.as_ref().unchecked_ref(),
        );
        before_install_prompt.forget();
        
        let app_installed = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            web_sys::console::log_1(&"🎉 PWA installed successfully".into());
        }) as Box<dyn FnMut(_)>);
        
        let _ = window.add_event_listener_with_callback(
            "appinstalled",
            app_installed.as_ref().unchecked_ref(),
        );
        app_installed.forget();
    }
}