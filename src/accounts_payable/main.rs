// src/accounts_payable/main.rs
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{get, post, put},
    Router,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type, Transaction, Postgres, postgres::PgPoolOptions};
use std::{sync::Arc, env};
use uuid::Uuid;
use tracing::{info, warn, error};

// Helper functions for extracting user info from headers
fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    headers
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

fn extract_company_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    headers
        .get("X-Company-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

// Database connection function for accounts payable service
async fn create_database_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("ACCOUNTS_PAYABLE_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("ACCOUNTS_PAYABLE_DATABASE_URL must be set"))?;

    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "20".to_string())
        .parse::<u32>()
        .unwrap_or(20);

    let min_connections = env::var("DB_MIN_CONNECTIONS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u32>()
        .unwrap_or(5);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .connect(&database_url)
        .await?;

    info!("Connected to accounts payable database");
    Ok(pool)
}

async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "invoice_status", rename_all = "SCREAMING_SNAKE_CASE")]
enum InvoiceStatus {
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
struct Vendor {
    id: Uuid,
    company_id: Uuid,
    vendor_code: String,
    vendor_name: String,
    npwp: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    payment_terms: i32,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VendorInvoice {
    id: Uuid,
    company_id: Uuid,
    vendor_id: Uuid,
    vendor_name: Option<String>,
    invoice_number: String,
    invoice_date: NaiveDate,
    due_date: NaiveDate,
    subtotal: Decimal,
    tax_amount: Decimal,
    total_amount: Decimal,
    paid_amount: Decimal,
    outstanding_amount: Decimal,
    status: InvoiceStatus,
    description: Option<String>,
    journal_entry_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateVendorRequest {
    company_id: Uuid,
    vendor_code: String,
    vendor_name: String,
    npwp: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    payment_terms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateVendorRequest {
    vendor_name: String,
    npwp: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    payment_terms: i32,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateVendorInvoiceRequest {
    company_id: Uuid,
    vendor_id: Uuid,
    invoice_number: String,
    invoice_date: NaiveDate,
    subtotal: Decimal,
    tax_amount: Decimal,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PaymentRequest {
    payment_amount: Decimal,
    payment_date: NaiveDate,
    payment_method: String,
    bank_account_id: Option<Uuid>,
    payment_reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgingReport {
    company_id: Uuid,
    report_date: NaiveDate,
    summary: AgingSummary,
    vendor_details: Vec<VendorAgingDetail>,
    generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgingSummary {
    current: Decimal,       // 0-30 days
    days_31_60: Decimal,
    days_61_90: Decimal,
    over_90_days: Decimal,
    total_outstanding: Decimal,
    invoice_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct VendorAgingDetail {
    vendor_id: Uuid,
    vendor_name: String,
    current: Decimal,
    days_31_60: Decimal,
    days_61_90: Decimal,
    over_90_days: Decimal,
    total_outstanding: Decimal,
    invoices: Vec<InvoiceAgingItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InvoiceAgingItem {
    invoice_id: Uuid,
    invoice_number: String,
    invoice_date: NaiveDate,
    due_date: NaiveDate,
    days_overdue: i32,
    outstanding_amount: Decimal,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

impl AppState {
    async fn log_vendor_activity(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        vendor_id: Uuid,
        activity: &str,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (id, table_name, record_id, action, new_values, user_id, timestamp)
            VALUES ($1, 'vendors', $2, $3, $4, $5, NOW())
            "#,
            Uuid::new_v4(),
            vendor_id,
            activity,
            serde_json::json!({"activity": activity}),
            user_id
        )
        .execute(&mut **tx)
        .await?;
        
        Ok(())
    }
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    let db_healthy = check_database_health(&state.db).await;
    
    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy { 
        StatusCode::OK 
    } else { 
        StatusCode::SERVICE_UNAVAILABLE 
    };
    
    (status_code, Json(serde_json::json!({
        "service": "accounts-payable-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    })))
}

async fn create_vendor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateVendorRequest>,
) -> Result<Json<Vendor>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    payload.company_id = company_id;
    
    let vendor_id = Uuid::new_v4();
    
    let mut tx = state.db.begin().await
    .map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if vendor code already exists
    let existing_vendor = sqlx::query!(
        "SELECT id FROM vendors WHERE company_id = $1 AND vendor_code = $2",
        company_id,
        payload.vendor_code
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to check existing vendor: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if existing_vendor.is_some() {
        return Err(StatusCode::CONFLICT);
    }
    
    let vendor = sqlx::query_as!(
        Vendor,
        r#"
        INSERT INTO vendors (id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, NOW(), NOW())
        RETURNING id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
        "#,
        vendor_id,
        payload.company_id,
        payload.vendor_code,
        payload.vendor_name,
        payload.npwp,
        payload.address,
        payload.phone,
        payload.email,
        payload.payment_terms.unwrap_or(30)
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to create vendor: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Log audit trail
    state.log_vendor_activity(&mut tx, vendor_id, "CREATE", user_id).await
        .map_err(|e| {
            error!("Failed to log vendor activity: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created vendor {} for company {}", vendor.vendor_code, company_id);

    Ok(Json(vendor))
}

async fn get_vendors(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<Vendor>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let include_inactive = params.get("include_inactive")
        .map(|v| v == "true")
        .unwrap_or(false);

    let search_term = params.get("search");

    let vendors = match (include_inactive, search_term) {
        (true, Some(search)) => {
            sqlx::query_as!(
                Vendor,
                r#"
                SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                FROM vendors 
                WHERE company_id = $1 
                  AND (vendor_name ILIKE $2 OR vendor_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                ORDER BY vendor_name
                "#,
                company_id,
                format!("%{}%", search)
            )
            .fetch_all(&state.db)
            .await
        }
        (false, Some(search)) => {
            sqlx::query_as!(
                Vendor,
                r#"
                SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                FROM vendors 
                WHERE company_id = $1 AND is_active = true
                  AND (vendor_name ILIKE $2 OR vendor_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                ORDER BY vendor_name
                "#,
                company_id,
                format!("%{}%", search)
            )
            .fetch_all(&state.db)
            .await
        }
        (true, None) => {
            sqlx::query_as!(
                Vendor,
                r#"
                SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                FROM vendors 
                WHERE company_id = $1
                ORDER BY vendor_name
                "#,
                company_id
            )
            .fetch_all(&state.db)
            .await
        }
        (false, None) => {
            sqlx::query_as!(
                Vendor,
                r#"
                SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                FROM vendors 
                WHERE company_id = $1 AND is_active = true
                ORDER BY vendor_name
                "#,
                company_id
            )
            .fetch_all(&state.db)
            .await
        }
    };

    let vendors = vendors.map_err(|e| {
        error!("Failed to fetch vendors: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(vendors))
}

async fn update_vendor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(vendor_id): Path<Uuid>,
    Json(payload): Json<UpdateVendorRequest>,
) -> Result<Json<Vendor>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let vendor = sqlx::query_as!(
        Vendor,
        r#"
        UPDATE vendors 
        SET vendor_name = $1, npwp = $2, address = $3, phone = $4, email = $5, 
            payment_terms = $6, is_active = $7, updated_at = NOW()
        WHERE id = $8 AND company_id = $9
        RETURNING id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
        "#,
        payload.vendor_name,
        payload.npwp,
        payload.address,
        payload.phone,
        payload.email,
        payload.payment_terms,
        payload.is_active,
        vendor_id,
        company_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update vendor: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Log audit trail
    state.log_vendor_activity(&mut tx, vendor_id, "UPDATE", user_id).await
        .map_err(|e| {
            error!("Failed to log vendor activity: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Updated vendor {} for company {}", vendor.vendor_code, company_id);

    Ok(Json(vendor))
}

async fn create_vendor_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateVendorInvoiceRequest>,
) -> Result<Json<VendorInvoice>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let _user_id = extract_user_id(&headers)?;
    payload.company_id = company_id;
    
    // Get vendor payment terms and validate vendor belongs to company
    let vendor = sqlx::query!(
        "SELECT payment_terms, vendor_name FROM vendors WHERE id = $1 AND company_id = $2 AND is_active = true", 
        payload.vendor_id,
        company_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch vendor: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::BAD_REQUEST)?;

    let due_date = payload.invoice_date + chrono::Duration::days(vendor.payment_terms as i64);
    let total_amount = payload.subtotal + payload.tax_amount;
    let invoice_id = Uuid::new_v4();

    // Check for duplicate invoice number
    let existing_invoice = sqlx::query!(
        "SELECT id FROM vendor_invoices WHERE company_id = $1 AND vendor_id = $2 AND invoice_number = $3",
        company_id,
        payload.vendor_id,
        payload.invoice_number
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to check existing invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if existing_invoice.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let invoice = sqlx::query!(
        r#"
        INSERT INTO vendor_invoices 
        (id, company_id, vendor_id, invoice_number, invoice_date, due_date, subtotal, tax_amount, total_amount, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
        RETURNING id, company_id, vendor_id, invoice_number, invoice_date, due_date,
                  subtotal, tax_amount, total_amount, paid_amount,
                  status as "status_str", description, journal_entry_id, created_at, updated_at
        "#,
        invoice_id,
        payload.company_id,
        payload.vendor_id,
        payload.invoice_number,
        payload.invoice_date,
        due_date,
        payload.subtotal,
        payload.tax_amount,
        total_amount,
        payload.description
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create vendor invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let status = invoice.status_str.as_deref()
        .and_then(|s| s.parse::<InvoiceStatus>().ok())
        .unwrap_or(InvoiceStatus::Draft);

    let vendor_invoice = VendorInvoice {
        id: invoice.id,
        company_id: invoice.company_id,
        vendor_id: invoice.vendor_id,
        vendor_name: Some(vendor.vendor_name),
        invoice_number: invoice.invoice_number,
        invoice_date: invoice.invoice_date,
        due_date: invoice.due_date,
        subtotal: invoice.subtotal,
        tax_amount: invoice.tax_amount,
        total_amount: invoice.total_amount,
        paid_amount: invoice.paid_amount,
        outstanding_amount: invoice.total_amount - invoice.paid_amount,
        status,
        description: invoice.description,
        journal_entry_id: invoice.journal_entry_id,
        created_at: invoice.created_at,
        updated_at: invoice.updated_at,
    };

    info!("Created vendor invoice {} for company {}", payload.invoice_number, company_id);

    Ok(Json(vendor_invoice))
}

async fn get_vendor_invoices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<VendorInvoice>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let status_filter = params.get("status");
    let vendor_id_filter = params.get("vendor_id")
        .and_then(|id| Uuid::parse_str(id).ok());

    let limit: i64 = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50)
        .min(200);

    let offset: i64 = params.get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    let invoices_data = match (status_filter, vendor_id_filter) {
        (Some(status), Some(vendor_id)) => {
            sqlx::query!(
                r#"
                SELECT vi.id, vi.company_id, vi.vendor_id, v.vendor_name, vi.invoice_number, 
                       vi.invoice_date, vi.due_date, vi.subtotal, vi.tax_amount, vi.total_amount, 
                       vi.paid_amount, vi.status as "status_str", vi.description, 
                       vi.journal_entry_id, vi.created_at, vi.updated_at
                FROM vendor_invoices vi
                LEFT JOIN vendors v ON vi.vendor_id = v.id
                WHERE vi.company_id = $1 AND vi.status = $2::invoice_status AND vi.vendor_id = $3
                ORDER BY vi.invoice_date DESC, vi.created_at DESC
                LIMIT $4 OFFSET $5
                "#,
                company_id,
                status,
                vendor_id,
                limit,
                offset
            )
            .fetch_all(&state.db)
            .await
        }
        (Some(status), None) => {
            sqlx::query!(
                r#"
                SELECT vi.id, vi.company_id, vi.vendor_id, v.vendor_name, vi.invoice_number, 
                       vi.invoice_date, vi.due_date, vi.subtotal, vi.tax_amount, vi.total_amount, 
                       vi.paid_amount, vi.status as "status_str", vi.description, 
                       vi.journal_entry_id, vi.created_at, vi.updated_at
                FROM vendor_invoices vi
                LEFT JOIN vendors v ON vi.vendor_id = v.id
                WHERE vi.company_id = $1 AND vi.status = $2::invoice_status
                ORDER BY vi.invoice_date DESC, vi.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                company_id,
                status,
                limit,
                offset
            )
            .fetch_all(&state.db)
            .await
        }
        (None, Some(vendor_id)) => {
            sqlx::query!(
                r#"
                SELECT vi.id, vi.company_id, vi.vendor_id, v.vendor_name, vi.invoice_number, 
                       vi.invoice_date, vi.due_date, vi.subtotal, vi.tax_amount, vi.total_amount, 
                       vi.paid_amount, vi.status as "status_str", vi.description, 
                       vi.journal_entry_id, vi.created_at, vi.updated_at
                FROM vendor_invoices vi
                LEFT JOIN vendors v ON vi.vendor_id = v.id
                WHERE vi.company_id = $1 AND vi.vendor_id = $2
                ORDER BY vi.invoice_date DESC, vi.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                company_id,
                vendor_id,
                limit,
                offset
            )
            .fetch_all(&state.db)
            .await
        }
        (None, None) => {
            sqlx::query!(
                r#"
                SELECT vi.id, vi.company_id, vi.vendor_id, v.vendor_name, vi.invoice_number, 
                       vi.invoice_date, vi.due_date, vi.subtotal, vi.tax_amount, vi.total_amount, 
                       vi.paid_amount, vi.status as "status_str", vi.description, 
                       vi.journal_entry_id, vi.created_at, vi.updated_at
                FROM vendor_invoices vi
                LEFT JOIN vendors v ON vi.vendor_id = v.id
                WHERE vi.company_id = $1
                ORDER BY vi.invoice_date DESC, vi.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                company_id,
                limit,
                offset
            )
            .fetch_all(&state.db)
            .await
        }
    };

    let invoices_data = invoices_data.map_err(|e| {
        error!("Failed to fetch vendor invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let invoices: Vec<VendorInvoice> = invoices_data
        .into_iter()
        .map(|row| {
            let status = row.status_str.as_deref()
                .and_then(|s| s.parse::<InvoiceStatus>().ok())
                .unwrap_or(InvoiceStatus::Draft);

            VendorInvoice {
                id: row.id,
                company_id: row.company_id,
                vendor_id: row.vendor_id,
                vendor_name: row.vendor_name,
                invoice_number: row.invoice_number,
                invoice_date: row.invoice_date,
                due_date: row.due_date,
                subtotal: row.subtotal,
                tax_amount: row.tax_amount,
                total_amount: row.total_amount,
                paid_amount: row.paid_amount,
                outstanding_amount: row.total_amount - row.paid_amount,
                status,
                description: row.description,
                journal_entry_id: row.journal_entry_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }
        })
        .collect();

    Ok(Json(invoices))
}

async fn pay_vendor_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<PaymentRequest>,
) -> Result<Json<VendorInvoice>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let _user_id = extract_user_id(&headers)?;

    if payload.payment_amount <= Decimal::ZERO {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get current invoice details
    let current_invoice = sqlx::query!(
        "SELECT total_amount, paid_amount FROM vendor_invoices WHERE id = $1 AND company_id = $2",
        invoice_id,
        company_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to fetch invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let remaining_amount = current_invoice.total_amount - current_invoice.paid_amount;
    
    if payload.payment_amount > remaining_amount {
        return Err(StatusCode::BAD_REQUEST);
    }

    let new_paid_amount = current_invoice.paid_amount + payload.payment_amount;
    let new_status = if new_paid_amount >= current_invoice.total_amount {
        InvoiceStatus::Paid
    } else {
        InvoiceStatus::Approved // Partial payment
    };

    let invoice_data = sqlx::query!(
        r#"
        UPDATE vendor_invoices 
        SET paid_amount = $1,
            status = $2::invoice_status,
            updated_at = NOW()
        WHERE id = $3 AND company_id = $4
        RETURNING id, company_id, vendor_id, invoice_number, invoice_date, due_date,
                  subtotal, tax_amount, total_amount, paid_amount,
                  status as "status_str", description, journal_entry_id, created_at, updated_at
        "#,
        new_paid_amount,
        new_status.to_string(),
        invoice_id,
        company_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update invoice payment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get vendor name
    let vendor_name = sqlx::query_scalar!(
        "SELECT vendor_name FROM vendors WHERE id = $1",
        invoice_data.vendor_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to fetch vendor name: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let status = invoice_data.status_str.as_deref()
        .and_then(|s| s.parse::<InvoiceStatus>().ok())
        .unwrap_or(InvoiceStatus::Draft);

    let invoice = VendorInvoice {
        id: invoice_data.id,
        company_id: invoice_data.company_id,
        vendor_id: invoice_data.vendor_id,
        vendor_name,
        invoice_number: invoice_data.invoice_number,
        invoice_date: invoice_data.invoice_date,
        due_date: invoice_data.due_date,
        subtotal: invoice_data.subtotal,
        tax_amount: invoice_data.tax_amount,
        total_amount: invoice_data.total_amount,
        paid_amount: invoice_data.paid_amount,
        outstanding_amount: invoice_data.total_amount - invoice_data.paid_amount,
        status,
        description: invoice_data.description,
        journal_entry_id: invoice_data.journal_entry_id,
        created_at: invoice_data.created_at,
        updated_at: invoice_data.updated_at,
    };

    info!("Processed payment of {} for invoice {}", payload.payment_amount, invoice.invoice_number);

    Ok(Json(invoice))
}

async fn get_aging_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AgingReport>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let as_of_date = params.get("as_of_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().naive_local().date());

    let aging_data = sqlx::query!(
        r#"
        SELECT 
            v.id as vendor_id,
            v.vendor_name,
            vi.id as invoice_id,
            vi.invoice_number,
            vi.invoice_date,
            vi.due_date,
            vi.total_amount - vi.paid_amount as outstanding_amount,
            $2 - vi.due_date as days_overdue
        FROM vendor_invoices vi
        JOIN vendors v ON vi.vendor_id = v.id
        WHERE vi.company_id = $1 
              AND vi.status != 'PAID'
              AND vi.total_amount > vi.paid_amount
        ORDER BY v.vendor_name, vi.due_date
        "#,
        company_id,
        as_of_date
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch aging data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut vendor_details: std::collections::HashMap<Uuid, VendorAgingDetail> = std::collections::HashMap::new();
    let mut summary = AgingSummary {
        current: Decimal::ZERO,
        days_31_60: Decimal::ZERO,
        days_61_90: Decimal::ZERO,
        over_90_days: Decimal::ZERO,
        total_outstanding: Decimal::ZERO,
        invoice_count: 0,
    };

    for row in aging_data {
        let outstanding = row.outstanding_amount.unwrap_or(Decimal::ZERO);
        let days_overdue = row.days_overdue.unwrap_or(0);
        
        // Update summary
        summary.total_outstanding += outstanding;
        summary.invoice_count += 1;

        match days_overdue {
            d if d <= 30 => summary.current += outstanding,
            d if d <= 60 => summary.days_31_60 += outstanding,
            d if d <= 90 => summary.days_61_90 += outstanding,
            _ => summary.over_90_days += outstanding,
        }

        // Update vendor detail
        let vendor_detail = vendor_details.entry(row.vendor_id).or_insert_with(|| {
            VendorAgingDetail {
                vendor_id: row.vendor_id,
                vendor_name: row.vendor_name.clone(),
                current: Decimal::ZERO,
                days_31_60: Decimal::ZERO,
                days_61_90: Decimal::ZERO,
                over_90_days: Decimal::ZERO,
                total_outstanding: Decimal::ZERO,
                invoices: Vec::new(),
            }
        });

        vendor_detail.total_outstanding += outstanding;
        match days_overdue {
            d if d <= 30 => vendor_detail.current += outstanding,
            d if d <= 60 => vendor_detail.days_31_60 += outstanding,
            d if d <= 90 => vendor_detail.days_61_90 += outstanding,
            _ => vendor_detail.over_90_days += outstanding,
        }

        vendor_detail.invoices.push(InvoiceAgingItem {
            invoice_id: row.invoice_id,
            invoice_number: row.invoice_number,
            invoice_date: row.invoice_date,
            due_date: row.due_date,
            days_overdue,
            outstanding_amount: outstanding,
        });
    }

    let report = AgingReport {
        company_id,
        report_date: as_of_date,
        summary,
        vendor_details: vendor_details.into_values().collect(),
        generated_at: chrono::Utc::now(),
    };

    info!("Generated aging report for company {} with {} vendors", company_id, report.vendor_details.len());

    Ok(Json(report))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,accounts_payable=debug")
        .init();

    info!("Starting Accounts Payable Service...");

    // Use the database pool creation function
    let pool = create_database_pool().await?;

    let app_state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/vendors", post(create_vendor))
        .route("/vendors", get(get_vendors))
        .route("/vendors/:id", put(update_vendor))
        .route("/invoices", post(create_vendor_invoice))
        .route("/invoices", get(get_vendor_invoices))
        .route("/invoices/:id/pay", put(pay_vendor_invoice))
        .route("/aging-report", get(get_aging_report))
        .with_state(app_state);

    let bind_addr = std::env::var("ACCOUNTS_PAYABLE_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3006".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Accounts Payable service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}