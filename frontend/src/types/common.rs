use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub npwp: String,
    pub address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub business_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub company_id: String,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}