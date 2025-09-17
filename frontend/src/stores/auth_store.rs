// frontend/src/stores/auth_store.rs
use leptos::*;

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub company_id: String,
}

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

#[derive(Clone)]
pub struct AuthStore {
    pub user: ReadSignal<Option<User>>,
    pub is_authenticated: ReadSignal<bool>,
}

pub fn create_auth_store() -> (ReadSignal<AuthState>, WriteSignal<AuthState>) {
    create_signal(AuthState::default())
}