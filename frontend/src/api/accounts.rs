use gloo_net::http::Request;
use crate::types::common::Account;
use crate::utils::storage;
use super::API_BASE;

pub async fn get_accounts() -> Result<Vec<Account>, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/accounts", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch accounts".to_string())
    }
}

pub async fn create_account(account_data: serde_json::Value) -> Result<Account, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::post(&format!("{}/accounts", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&account_data)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to create account".to_string())
    }
}