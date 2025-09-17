// frontend/src/models.rs
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String, // Changed from Uuid to String
    pub email: String,
    pub full_name: String,
    pub company_id: String, // Changed from Uuid to String
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String, // Changed from Uuid to String
    pub name: String,
    pub npwp: String,
    pub address: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String, // Changed from Uuid to String
    pub company_id: String, // Changed from Uuid to String
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryFormData {
    pub entry_date: String,
    pub description: String,
    pub reference: String,
    pub lines: Vec<JournalEntryLineFormData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryLineFormData {
    pub account_id: String,
    pub description: String,
    pub debit_amount: String,
    pub credit_amount: String,
}

impl Default for JournalEntryFormData {
    fn default() -> Self {
        Self {
            entry_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            description: String::new(),
            reference: String::new(),
            lines: vec![
                JournalEntryLineFormData::default(),
                JournalEntryLineFormData::default(),
            ],
        }
    }
}

impl Default for JournalEntryLineFormData {
    fn default() -> Self {
        Self {
            account_id: String::new(),
            description: String::new(),
            debit_amount: String::new(),
            credit_amount: String::new(),
        }
    }
}