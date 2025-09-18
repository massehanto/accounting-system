use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    pub created_at: String,
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