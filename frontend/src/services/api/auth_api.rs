// frontend/src/services/api/auth_api.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub company_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

pub struct AuthApi;

impl AuthApi {
    pub async fn login(credentials: LoginCredentials) -> Result<(), ApiError> {
        // Implementation would go here
        Ok(())
    }

    pub async fn logout() -> Result<(), ApiError> {
        // Implementation would go here
        Ok(())
    }
}