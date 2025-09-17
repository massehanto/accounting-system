use chrono::{NaiveDate, DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheetReport {
    pub company_id: Uuid,
    pub as_of_date: NaiveDate,
    pub assets: BalanceSheetSection,
    pub liabilities: BalanceSheetSection,
    pub equity: BalanceSheetSection,
    pub total_assets: Decimal,
    pub total_liabilities_and_equity: Decimal,
    pub is_balanced: bool,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheetSection {
    pub title: String,
    pub total: Decimal,
    pub subsections: HashMap<String, BalanceSheetSubsection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheetSubsection {
    pub title: String,
    pub accounts: Vec<AccountItem>,
    pub total: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountItem {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub balance: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeStatementReport {
    pub company_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub revenue: IncomeStatementSection,
    pub expenses: IncomeStatementSection,
    pub gross_profit: Decimal,
    pub net_income: Decimal,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeStatementSection {
    pub title: String,
    pub total: Decimal,
    pub subsections: HashMap<String, IncomeStatementSubsection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeStatementSubsection {
    pub title: String,
    pub accounts: Vec<AccountItem>,
    pub total: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialBalanceReport {
    pub company_id: Uuid,
    pub as_of_date: NaiveDate,
    pub accounts: Vec<TrialBalanceAccount>,
    pub total_debits: Decimal,
    pub total_credits: Decimal,
    pub is_balanced: bool,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialBalanceAccount {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub debit_balance: Decimal,
    pub credit_balance: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxReport {
    pub company_id: Uuid,
    pub tax_type: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub transactions: Vec<TaxReportTransaction>,
    pub total_tax_base: Decimal,
    pub total_tax_amount: Decimal,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxReportTransaction {
    pub transaction_id: Uuid,
    pub transaction_date: NaiveDate,
    pub vendor_name: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_number: Option<String>,
    pub tax_base_amount: Decimal,
    pub tax_amount: Decimal,
    pub tax_invoice_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportExportRequest {
    pub report_type: String,
    pub format: ExportFormat,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExportFormat {
    Pdf,
    Excel,
    Csv,
}