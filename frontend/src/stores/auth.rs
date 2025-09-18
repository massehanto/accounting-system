use leptos::*;
use crate::types::AuthState;

#[derive(Clone)]
pub struct AuthStore {
    pub state: ReadSignal<AuthState>,
    pub set_state: WriteSignal<AuthState>,
}

pub fn create_auth_store() -> (ReadSignal<AuthState>, WriteSignal<AuthState>) {
    create_signal(AuthState::default())
}