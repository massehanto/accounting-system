// frontend/src/api.rs
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::utils;

const API_BASE: &str = "/api";

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

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JournalEntryStatus {
    Draft,
    PendingApproval,
    Approved,
    Posted,
    Cancelled,
}

impl std::fmt::Display for JournalEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JournalEntryStatus::Draft => write!(f, "Draft"),
            JournalEntryStatus::PendingApproval => write!(f, "Pending Approval"),
            JournalEntryStatus::Approved => write!(f, "Approved"),
            JournalEntryStatus::Posted => write!(f, "Posted"),
            JournalEntryStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JournalEntry {
    pub id: String,
    pub company_id: String,
    pub entry_number: String,
    pub entry_date: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub total_debit: f64,
    pub total_credit: f64,
    pub status: JournalEntryStatus,
    pub is_posted: bool,
    pub created_by: String,
    pub approved_by: Option<String>,
    pub posted_by: Option<String>,
    pub created_at: String,
    pub approved_at: Option<String>,
    pub posted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JournalEntryLine {
    pub id: String,
    pub journal_entry_id: String,
    pub account_id: String,
    pub account_code: Option<String>,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub debit_amount: f64,
    pub credit_amount: f64,
    pub line_number: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JournalEntryWithLines {
    pub journal_entry: JournalEntry,
    pub lines: Vec<JournalEntryLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub company_id: String,
    pub entry_date: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub lines: Vec<CreateJournalEntryLineRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJournalEntryLineRequest {
    pub account_id: String,
    pub description: Option<String>,
    pub debit_amount: f64,
    pub credit_amount: f64,
}

pub async fn login(email: &str, password: &str) -> Result<LoginResponse, String> {
    let request = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let response = Request::post(&format!("{}/auth/login", API_BASE))
        .json(&request)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let login_response: LoginResponse = response.json().await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        utils::set_token(&login_response.token);
        utils::set_user_info(&login_response.user_id, &login_response.company_id);
        
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
    let response = Request::get(&format!("{}/auth/verify?token={}", API_BASE, token))
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

pub async fn get_companies() -> Result<Vec<serde_json::Value>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
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

pub async fn get_accounts() -> Result<Vec<serde_json::Value>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
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

pub async fn get_journal_entries(status_filter: Option<&str>) -> Result<Vec<JournalEntry>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
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

pub async fn get_journal_entry_with_lines(id: &str) -> Result<JournalEntryWithLines, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/journal-entries/{}", API_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch journal entry".to_string())
    }
}

pub async fn create_journal_entry(entry: CreateJournalEntryRequest) -> Result<JournalEntryWithLines, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let response = Request::post(&format!("{}/journal-entries", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&entry)
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
    let token = utils::get_token().ok_or("No token available")?;
    
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
            _ => {
                if let Ok(error) = response.json::<ApiError>().await {
                    Err(format!("Failed to update status: {}", error.message))
                } else {
                    Err(format!("Failed to update status with code: {}", status))
                }
            }
        }
    }
}

pub async fn delete_journal_entry(id: &str) -> Result<(), String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let response = Request::delete(&format!("{}/journal-entries/{}", API_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() || response.status() == 204 {
        Ok(())
    } else {
        let status = response.status();
        match status {
            409 => Err("Cannot delete posted or approved journal entry".to_string()),
            404 => Err("Journal entry not found".to_string()),
            _ => Err(format!("Failed to delete journal entry with status: {}", status))
        }
    }
}

pub async fn get_trial_balance(as_of_date: Option<&str>) -> Result<serde_json::Value, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
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

pub async fn get_account_balances(as_of_date: Option<&str>, account_id: Option<&str>) -> Result<Vec<serde_json::Value>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let mut url = format!("{}/account-balances", API_BASE);
    let mut params = Vec::new();
    
    if let Some(date) = as_of_date {
        params.push(format!("as_of_date={}", date));
    }
    if let Some(id) = account_id {
        params.push(format!("account_id={}", id));
    }
    
    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
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
        Err("Failed to fetch account balances".to_string())
    }
}

// Vendor API functions
pub async fn get_vendors() -> Result<Vec<serde_json::Value>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
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
    let token = utils::get_token().ok_or("No token available")?;
    
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

// Customer API functions
pub async fn get_customers() -> Result<Vec<serde_json::Value>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/customers", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch customers".to_string())
    }
}

pub async fn create_customer(customer_data: serde_json::Value) -> Result<serde_json::Value, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let response = Request::post(&format!("{}/customers", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&customer_data)
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
            Err(format!("Failed to create customer: {}", error.message))
        } else {
            Err(format!("Failed to create customer with status: {}", status))
        }
    }
}

// Inventory API functions
pub async fn get_inventory_items() -> Result<Vec<serde_json::Value>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let response = Request::get(&format!("{}/items", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch inventory items".to_string())
    }
}

pub async fn create_inventory_item(item_data: serde_json::Value) -> Result<serde_json::Value, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let response = Request::post(&format!("{}/items", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&item_data)
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
            Err(format!("Failed to create inventory item: {}", error.message))
        } else {
            Err(format!("Failed to create inventory item with status: {}", status))
        }
    }
}

// Tax API functions
pub async fn get_tax_configurations() -> Result<Vec<serde_json::Value>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
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
    let token = utils::get_token().ok_or("No token available")?;
    
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

// Reporting API functions
pub async fn get_balance_sheet(period_end: &str) -> Result<serde_json::Value, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
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
    let token = utils::get_token().ok_or("No token available")?;
    
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
    let token = utils::get_token().ok_or("No token available")?;
    
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

// Utility function to handle common error responses
pub fn handle_api_error(status: u16, error_msg: Option<String>) -> String {
    match status {
        400 => error_msg.unwrap_or_else(|| "Bad request - invalid data provided".to_string()),
        401 => "Unauthorized - please login again".to_string(),
        403 => "Forbidden - insufficient permissions".to_string(),
        404 => error_msg.unwrap_or_else(|| "Resource not found".to_string()),
        409 => error_msg.unwrap_or_else(|| "Conflict - operation not allowed in current state".to_string()),
        422 => error_msg.unwrap_or_else(|| "Invalid data - please check your input".to_string()),
        500 => "Internal server error - please try again later".to_string(),
        502 => "Service temporarily unavailable".to_string(),
        503 => "Service unavailable - maintenance in progress".to_string(),
        _ => error_msg.unwrap_or_else(|| format!("Unexpected error ({})", status)),
    }
}

// Helper function for pagination
pub struct PaginationParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl PaginationParams {
    pub fn new(limit: Option<usize>, offset: Option<usize>) -> Self {
        Self { limit, offset }
    }
    
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        
        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

// Enhanced journal entry functions with pagination
pub async fn get_journal_entries_paginated(
    status_filter: Option<&str>,
    pagination: Option<PaginationParams>
) -> Result<Vec<JournalEntry>, String> {
    let token = utils::get_token().ok_or("No token available")?;
    
    let mut params = Vec::new();
    
    if let Some(status) = status_filter {
        params.push(format!("status={}", status));
    }
    
    if let Some(p) = pagination {
        if let Some(limit) = p.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = p.offset {
            params.push(format!("offset={}", offset));
        }
    }
    
    let url = if params.is_empty() {
        format!("{}/journal-entries", API_BASE)
    } else {
        format!("{}/journal-entries?{}", API_BASE, params.join("&"))
    };
    
    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response.json().await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err(handle_api_error(response.status(), None))
    }
}