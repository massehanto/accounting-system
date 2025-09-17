// frontend/src/lib.rs
use leptos::*;

mod components;
mod pages;
mod api;
mod models;
mod utils;
mod app;

pub use components::*;
pub use pages::*;
pub use models::*;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <app::App/> })
}