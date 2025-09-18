use leptos::*;
use wasm_bindgen::prelude::*;

pub mod app;
pub mod components;
pub mod pages;
pub mod api;
pub mod types;
pub mod utils;
pub mod stores;
pub mod hooks;
pub mod config;

pub use app::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    if let Some(window) = web_sys::window() {
        if let Ok(navigator) = window.navigator().service_worker() {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = navigator.register("/sw.js").await {
                    web_sys::console::error_1(&format!("SW registration failed: {:?}", e).into());
                }
            });
        }
    }

    mount_to_body(|| view! { <App/> })
}