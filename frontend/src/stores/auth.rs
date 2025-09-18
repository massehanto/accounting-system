use leptos::*;
use crate::types::auth::{User, AuthState};
use crate::utils::storage;

#[derive(Clone)]
pub struct AuthStore {
    state: RwSignal<AuthState>,
}

impl AuthStore {
    pub fn new() -> Self {
        let initial_state = AuthState::default();
        Self {
            state: create_rw_signal(initial_state),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.state.get().is_authenticated
    }

    pub fn get_user(&self) -> Option<User> {
        self.state.get().user
    }

    pub fn set_user(&self, user: User, token: String) {
        storage::set_token(&token);
        storage::set_user_info(&user.id, &user.company_id);
        
        self.state.update(|state| {
            state.user = Some(user);
            state.token = Some(token);
            state.is_authenticated = true;
        });
    }

    pub fn logout(&self) {
        storage::clear_user_data();
        self.state.set(AuthState::default());
    }
}

pub fn provide_auth_store() {
    provide_context(AuthStore::new());
}