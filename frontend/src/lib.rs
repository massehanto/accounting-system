// frontend/src/lib.rs
use leptos::*;

mod components;
mod pages;
mod api;
mod models;
mod utils;
mod error;
mod stores;
mod app;

pub use components::*;
pub use pages::*;
pub use models::*;
pub use error::*;
pub use stores::*;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Register service worker for PWA
    if let Some(navigator) = web_sys::window().and_then(|w| w.navigator().service_worker().ok()) {
        wasm_bindgen_futures::spawn_local(async move {
            let _ = navigator.register("./sw.js").await;
        });
    }
    
    mount_to_body(|| view! { <app::App/> })
}