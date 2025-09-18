pub mod auth;
pub mod journal;
pub mod accounts;
pub mod companies;
pub mod vendors;
pub mod customers;
pub mod inventory;
pub mod tax;
pub mod reports;

pub use auth::*;
pub use journal::*;
pub use accounts::*;
pub use companies::*;
pub use vendors::*;
pub use customers::*;
pub use inventory::*;
pub use tax::*;
pub use reports::*;

use serde::{Deserialize, Serialize};

const API_BASE: &str = "/api";

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

// Helper function to handle common error responses
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