// frontend/src/stores/app_store.rs
use leptos::*;

#[derive(Clone, Debug)]
pub struct AppState {
    pub loading: bool,
    pub error: Option<String>,
    pub theme: String,
    pub sidebar_open: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            loading: false,
            error: None,
            theme: "light".to_string(),
            sidebar_open: true,
        }
    }
}

pub fn create_app_store() -> (ReadSignal<AppState>, WriteSignal<AppState>) {
    create_signal(AppState::default())
}