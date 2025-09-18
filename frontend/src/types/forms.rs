use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CompanyFormData {
    pub name: String,
    pub npwp: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub business_type: String,
}

impl Default for CompanyFormData {
    fn default() -> Self {
        Self {
            name: String::new(),
            npwp: String::new(),
            address: String::new(),
            phone: String::new(),
            email: String::new(),
            business_type: String::new(),
        }
    }
}