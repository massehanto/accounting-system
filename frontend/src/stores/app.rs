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

#[derive(Clone)]
pub struct AppStore {
    state: RwSignal<AppState>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            state: create_rw_signal(AppState::default()),
        }
    }

    pub fn set_loading(&self, loading: bool) {
        self.state.update(|state| state.loading = loading);
    }

    pub fn set_error(&self, error: Option<String>) {
        self.state.update(|state| state.error = error);
    }

    pub fn toggle_sidebar(&self) {
        self.state.update(|state| state.sidebar_open = !state.sidebar_open);
    }

    pub fn is_sidebar_open(&self) -> bool {
        self.state.get().sidebar_open
    }
}

pub fn provide_app_store() {
    provide_context(AppStore::new());
}