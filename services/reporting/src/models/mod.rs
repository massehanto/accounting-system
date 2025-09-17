use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancialReport {
    pub company_id: Uuid,
    pub report_type: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub generated_by: Uuid,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheetData {
    pub assets: BalanceSheetSection,
    pub liabilities: BalanceSheetSection,
    pub equity: BalanceSheetSection,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub total_equity: Decimal,
    pub is_balanced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSheetSection {
    pub current: HashMap<String, Decimal>,
    pub non_current: HashMap<String, Decimal>,
    pub total: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeStatementData {
    pub revenue: HashMap<String, Decimal>,
    pub cost_of_goods_sold: HashMap<String, Decimal>,
    pub gross_profit: Decimal,
    pub operating_expenses: HashMap<String, Decimal>,
    pub operating_income: Decimal,
    pub other_income: HashMap<String, Decimal>,
    pub other_expenses: HashMap<String, Decimal>,
    pub net_income_before_tax: Decimal,
    pub tax_expense: Decimal,
    pub net_income: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CashFlowData {
    pub operating_activities: HashMap<String, Decimal>,
    pub investing_activities: HashMap<String, Decimal>,
    pub financing_activities: HashMap<String, Decimal>,
    pub net_cash_from_operations: Decimal,
    pub net_cash_from_investing: Decimal,
    pub net_cash_from_financing: Decimal,
    pub net_change_in_cash: Decimal,
    pub beginning_cash: Decimal,
    pub ending_cash: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialBalanceData {
    pub accounts: Vec<TrialBalanceAccount>,
    pub total_debits: Decimal,
    pub total_credits: Decimal,
    pub is_balanced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrialBalanceAccount {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub debit_balance: Decimal,
    pub credit_balance: Decimal,
}