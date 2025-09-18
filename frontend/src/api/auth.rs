use gloo_net::http::Request;
use serde::{Serialize, Deserialize};
use crate::types::auth::{LoginRequest, LoginResponse};
use crate::utils::storage;

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

pub async fn login(email: &str, password: &str) -> Result<LoginResponse, String> {
    let request = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let response = Request::post("/api/auth/login")
        .json(&request)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let login_response: LoginResponse = response.json().await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        storage::set_token(&login_response.token);
        storage::set_user_info(&login_response.user_id, &login_response.company_id);
        
        Ok(login_response)
    } else {
        let status = response.status();
        if let Ok(error) = response.json::<ApiError>().await {
            Err(format!("Login failed: {}", error.message))
        } else {
            Err(format!("Login failed with status: {}", status))
        }
    }
}

pub async fn verify_token(token: &str) -> Result<serde_json::Value, String> {
    let response = Request::get(&format!("/api/auth/verify?token={}", token))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Token verification failed".to_string())
    }
}