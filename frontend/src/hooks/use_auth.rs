// src/hooks/use_auth.rs
use leptos::*;
use crate::stores::AuthStore;
use crate::services::api::AuthApi;

pub fn use_auth() -> AuthHook {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    
    AuthHook {
        user: auth_store.user,
        is_authenticated: auth_store.is_authenticated,
        login: create_action(move |credentials: &LoginCredentials| {
            let credentials = credentials.clone();
            async move {
                AuthApi::login(credentials).await
            }
        }),
        logout: create_action(move |_: &()| async move {
            AuthApi::logout().await
        }),
    }
}

pub struct AuthHook {
    pub user: ReadSignal<Option<User>>,
    pub is_authenticated: ReadSignal<bool>,
    pub login: Action<LoginCredentials, Result<(), ApiError>>,
    pub logout: Action<(), Result<(), ApiError>>,
}