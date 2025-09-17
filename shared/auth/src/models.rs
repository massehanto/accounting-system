use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub company_id: Uuid,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub email: String,
    pub company_id: String,
    pub full_name: String,
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub jti: String,      // JWT ID (for refresh tokens)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub jti: String,      // JWT ID
    pub user_id: String,
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: AuthUser,
    pub permissions: Vec<String>,
    pub token_jti: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}