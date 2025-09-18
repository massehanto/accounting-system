use gloo_net::http::Request;
use crate::utils::storage;
use super::API_BASE;

pub async fn get_tax_configurations() -> Result<Vec<serde_json::Value>, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/tax-configurations", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch tax configurations".to_string())
    }
}

pub async fn get_tax_report(tax_type: &str, period_start: &str, period_end: &str) -> Result<serde_json::Value, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let url = format!("{}/tax-report?tax_type={}&period_start={}&period_end={}", 
        API_BASE, tax_type, period_start, period_end);
    
    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch tax report".to_string())
    }
}