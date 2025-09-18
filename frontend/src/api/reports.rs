use gloo_net::http::Request;
use crate::utils::storage;
use super::API_BASE;

pub async fn get_balance_sheet(period_end: &str) -> Result<serde_json::Value, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/reports/balance-sheet?period_end={}", API_BASE, period_end))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch balance sheet".to_string())
    }
}

pub async fn get_income_statement(period_start: &str, period_end: &str) -> Result<serde_json::Value, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let url = format!("{}/reports/income-statement?period_start={}&period_end={}", 
        API_BASE, period_start, period_end);
    
    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch income statement".to_string())
    }
}

pub async fn get_cash_flow_statement(period_start: &str, period_end: &str) -> Result<serde_json::Value, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let url = format!("{}/reports/cash-flow?period_start={}&period_end={}", 
        API_BASE, period_start, period_end);
    
    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch cash flow statement".to_string())
    }
}

pub async fn get_trial_balance(as_of_date: Option<&str>) -> Result<serde_json::Value, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let mut url = format!("{}/trial-balance", API_BASE);
    if let Some(date) = as_of_date {
        url = format!("{}?as_of_date={}", url, date);
    }
    
    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch trial balance".to_string())
    }
}