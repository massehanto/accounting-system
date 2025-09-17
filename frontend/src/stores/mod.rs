// frontend/src/stores/mod.rs
pub mod auth_store;
pub mod app_store;

pub use auth_store::*;
pub use app_store::*;

// frontend/src/stores/auth_store.rs
use leptos::*;
use crate::models::User;

#[derive(Clone, Debug)]
pub struct AuthState {
    pub user: Option<User>,
    pub token: Option<String>,
    pub company_id: Option<String>,
    pub is_authenticated: bool,
    pub is_loading: bool,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            user: None,
            token: None,
            company_id: None,
            is_authenticated: false,
            is_loading: false,
        }
    }
}

pub fn create_auth_store() -> (ReadSignal<AuthState>, WriteSignal<AuthState>) {
    create_signal(AuthState::default())
}