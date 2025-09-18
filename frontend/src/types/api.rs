use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: String,
    pub company_id: String,
    pub vendor_code: String,
    pub vendor_name: String,
    pub npwp: Option<String>,
    pub address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub company_id: String,
    pub customer_code: String,
    pub customer_name: String,
    pub npwp: Option<String>,
    pub address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub credit_limit: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub company_id: String,
    pub item_code: String,
    pub item_name: String,
    pub item_type: String,
    pub unit_cost: f64,
    pub selling_price: f64,
    pub quantity_on_hand: f64,
    pub reorder_level: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxConfiguration {
    pub id: String,
    pub company_id: String,
    pub tax_type: String,
    pub tax_rate: f64,
    pub effective_date: String,
    pub end_date: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
}