use gloo_net::http::Request;
use crate::types::common::ApiError;
use crate::utils::storage;
use super::API_BASE;

pub async fn get_vendors() -> Result<Vec<serde_json::Value>, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/vendors", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch vendors".to_string())
    }
}

pub async fn create_vendor(vendor_data: serde_json::Value) -> Result<serde_json::Value, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::post(&format!("{}/vendors", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&vendor_data)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let status = response.status();
        if let Ok(error) = response.json::<ApiError>().await {
            Err(format!("Failed to create vendor: {}", error.message))
        } else {
            Err(format!("Failed to create vendor with status: {}", status))
        }
    }
}