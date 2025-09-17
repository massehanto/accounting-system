use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use validator::Validate;

// Reuse InvoiceStatus from accounts-payable
#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "invoice_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
    Draft,
    Pending,
    Approved,
    Paid,
    Cancelled,
}

impl std::str::FromStr for InvoiceStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DRAFT" => Ok(InvoiceStatus::Draft),
            "PENDING" => Ok(InvoiceStatus::Pending),
            "APPROVED" => Ok(InvoiceStatus::Approved),
            "PAID" => Ok(InvoiceStatus::Paid),
            "CANCELLED" => Ok(InvoiceStatus::Cancelled),
            _ => Err(format!("Invalid invoice status: {}", s))
        }
    }
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "DRAFT"),
            InvoiceStatus::Pending => write!(f, "PENDING"),
            InvoiceStatus::Approved => write!(f, "APPROVED"),
            InvoiceStatus::Paid => write!(f, "PAID"),
            InvoiceStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub company_id: Uuid,
    pub customer_code: String,
    pub customer_name: String,
    pub npwp: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub credit_limit: Decimal,
    pub payment_terms: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerInvoice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub paid_amount: Decimal,
    pub outstanding_amount: Decimal,
    pub status: InvoiceStatus,
    pub description: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerPayment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub company_id: Uuid,
    pub payment_amount: Decimal,
    pub payment_date: NaiveDate,
    pub payment_method: String,
    pub bank_account_id: Option<Uuid>,
    pub payment_reference: Option<String>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerCreditInfo {
    pub customer_id: Uuid,
    pub customer_name: String,
    pub credit_limit: Decimal,
    pub current_outstanding: Decimal,
    pub available_credit: Decimal,
    pub credit_utilization: f64, // Percentage
    pub outstanding_invoices: u32,
    pub days_past_due: Option<i32>,
    pub credit_status: String, // GOOD, HIGH_UTILIZATION, OVER_LIMIT
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    pub company_id: Uuid,
    #[validate(length(min = 1, max = 20, message = "Customer code must be 1-20 characters"))]
    pub customer_code: String,
    #[validate(length(min = 1, max = 255, message = "Customer name must be 1-255 characters"))]
    pub customer_name: String,
    pub npwp: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    pub credit_limit: Option<Decimal>,
    pub payment_terms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateCustomerRequest {
    #[validate(length(min = 1, max = 255, message = "Customer name must be 1-255 characters"))]
    pub customer_name: String,
    pub npwp: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    pub credit_limit: Decimal,
    pub payment_terms: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCustomerInvoiceRequest {
    pub company_id: Uuid,
    pub customer_id: Uuid,
    #[validate(length(min = 1, max = 50, message = "Invoice number must be 1-50 characters"))]
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    #[validate(range(min = 0, message = "Subtotal cannot be negative"))]
    pub subtotal: Decimal,
    #[validate(range(min = 0, message = "Tax amount cannot be negative"))]
    pub tax_amount: Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PaymentRequest {
    #[validate(range(min = 0.01, message = "Payment amount must be positive"))]
    pub payment_amount: Decimal,
    pub payment_date: NaiveDate,
    pub payment_method: String,
    pub bank_account_id: Option<Uuid>,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerAgingReport {
    pub company_id: Uuid,
    pub report_date: NaiveDate,
    pub summary: AgingSummary,
    pub customer_details: Vec<CustomerAgingDetail>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgingSummary {
    pub current: Decimal,       // 0-30 days
    pub days_31_60: Decimal,
    pub days_61_90: Decimal,
    pub over_90_days: Decimal,
    pub total_outstanding: Decimal,
    pub invoice_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerAgingDetail {
    pub customer_id: Uuid,
    pub customer_name: String,
    pub credit_limit: Decimal,
    pub current: Decimal,
    pub days_31_60: Decimal,
    pub days_61_90: Decimal,
    pub over_90_days: Decimal,
    pub total_outstanding: Decimal,
    pub credit_utilization: f64,
    pub invoices: Vec<InvoiceAgingItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceAgingItem {
    pub invoice_id: Uuid,
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub days_overdue: i32,
    pub outstanding_amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerStatistics {
    pub customer_id: Uuid,
    pub total_invoices: u32,
    pub total_sales: Decimal,
    pub total_payments: Decimal,
    pub outstanding_amount: Decimal,
    pub average_invoice_amount: Decimal,
    pub average_payment_days: f64,
    pub last_invoice_date: Option<NaiveDate>,
    pub last_payment_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceFilters {
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
}