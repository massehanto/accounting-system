use gloo_net::http::Request;
use crate::types::journal::{JournalEntry, JournalEntryStatus};
use crate::types::common::ApiError;
use crate::utils::storage;
use super::API_BASE;

pub async fn get_journal_entries(status_filter: Option<&str>) -> Result<Vec<JournalEntry>, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let mut url = format!("{}/journal-entries", API_BASE);
    if let Some(status) = status_filter {
        url = format!("{}?status={}", url, status);
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
        Err("Failed to fetch journal entries".to_string())
    }
}

pub async fn create_journal_entry(entry_data: serde_json::Value) -> Result<JournalEntry, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::post(&format!("{}/journal-entries", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&entry_data)
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
            Err(format!("Failed to create journal entry: {}", error.message))
        } else {
            Err(format!("Failed to create journal entry with status: {}", status))
        }
    }
}

pub async fn update_journal_entry_status(id: &str, new_status: &str) -> Result<JournalEntry, String> {
    let token = storage::get_token().ok_or("No token available")?;
    
    let response = Request::put(&format!("{}/journal-entries/{}/status?status={}", API_BASE, id, new_status))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let status = response.status();
        match status {
            409 => Err("Invalid status transition".to_string()),
            404 => Err("Journal entry not found".to_string()),
            _ => Err(format!("Failed to update status with code: {}", status))
        }
    }
}