use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub company_id: String,
    pub is_active: bool,
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

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub company_id: String,
    pub expires_in: i64,
}