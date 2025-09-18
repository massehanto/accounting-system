use gloo_net::http::Request;
use crate::types::common::Company;
use crate::utils::storage;
use super::API_BASE;

pub async fn get_companies() -> Result<Vec<Company>, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/companies", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch companies".to_string())
    }
}

pub async fn update_company(id: &str, company_data: serde_json::Value) -> Result<Company, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::put(&format!("{}/companies/{}", API_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&company_data)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to update company".to_string())
    }
}