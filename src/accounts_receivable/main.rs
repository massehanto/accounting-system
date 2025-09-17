// src/accounts_receivable/main.rs
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

// Database connection function specific to accounts receivable service
async fn create_database_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("ACCOUNTS_RECEIVABLE_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("ACCOUNTS_RECEIVABLE_DATABASE_URL must be set"))?;

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

    info!("Connected to accounts receivable database: {}", database_url.split('@').last().unwrap_or("unknown"));
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
struct Customer {
    id: Uuid,
    company_id: Uuid,
    customer_code: String,
    customer_name: String,
    npwp: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    credit_limit: Decimal,
    payment_terms: i32,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomerInvoice {
    id: Uuid,
    company_id: Uuid,
    customer_id: Uuid,
    customer_name: Option<String>,
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
struct CreateCustomerRequest {
    company_id: Uuid,
    customer_code: String,
    customer_name: String,
    npwp: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    credit_limit: Option<Decimal>,
    payment_terms: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateCustomerRequest {
    customer_name: String,
    npwp: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    credit_limit: Decimal,
    payment_terms: i32,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateCustomerInvoiceRequest {
    company_id: Uuid,
    customer_id: Uuid,
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
struct CustomerAgingReport {
    company_id: Uuid,
    report_date: NaiveDate,
    summary: AgingSummary,
    customer_details: Vec<CustomerAgingDetail>,
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
struct CustomerAgingDetail {
    customer_id: Uuid,
    customer_name: String,
    credit_limit: Decimal,
    current: Decimal,
    days_31_60: Decimal,
    days_61_90: Decimal,
    over_90_days: Decimal,
    total_outstanding: Decimal,
    credit_utilization: f64,
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
    async fn log_customer_activity(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        customer_id: Uuid,
        activity: &str,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (id, table_name, record_id, action, new_values, user_id, timestamp)
            VALUES ($1, 'customers', $2, $3, $4, $5, NOW())
            "#,
            Uuid::new_v4(),
            customer_id,
            activity,
            serde_json::json!({"activity": activity}),
            user_id
        )
        .execute(&mut **tx)
        .await?;
        
        Ok(())
    }

    async fn check_credit_limit(
        &self,
        customer_id: Uuid,
        additional_amount: Decimal,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT 
                c.credit_limit,
                COALESCE(SUM(ci.total_amount - ci.paid_amount), 0) as current_outstanding
            FROM customers c
            LEFT JOIN customer_invoices ci ON c.id = ci.customer_id 
                AND ci.status != 'CANCELLED' AND ci.total_amount > ci.paid_amount
            WHERE c.id = $1
            GROUP BY c.id, c.credit_limit
            "#,
            customer_id
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = result {
            let current_outstanding = row.current_outstanding.unwrap_or(Decimal::ZERO);
            let available_credit = row.credit_limit - current_outstanding;
            Ok(available_credit >= additional_amount)
        } else {
            Ok(false) // Customer not found
        }
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
        "service": "accounts-receivable-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    })))
}

async fn create_customer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    payload.company_id = company_id;
    
    let customer_id = Uuid::new_v4();
    
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if customer code already exists
    let existing_customer = sqlx::query!(
        "SELECT id FROM customers WHERE company_id = $1 AND customer_code = $2",
        company_id,
        payload.customer_code
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to check existing customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if existing_customer.is_some() {
        return Err(StatusCode::CONFLICT);
    }
    
    let customer = sqlx::query_as!(
        Customer,
        r#"
        INSERT INTO customers (id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())
        RETURNING id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
        "#,
        customer_id,
        payload.company_id,
        payload.customer_code,
        payload.customer_name,
        payload.npwp,
        payload.address,
        payload.phone,
        payload.email,
        payload.credit_limit.unwrap_or(Decimal::from(0)),
        payload.payment_terms.unwrap_or(30)
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to create customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Log audit trail
    state.log_customer_activity(&mut tx, customer_id, "CREATE", user_id).await
        .map_err(|e| {
            error!("Failed to log customer activity: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created customer {} for company {}", customer.customer_code, company_id);

    Ok(Json(customer))
}

async fn get_customers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<Customer>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let include_inactive = params.get("include_inactive")
        .map(|v| v == "true")
        .unwrap_or(false);

    let search_term = params.get("search");

    let customers = match (include_inactive, search_term) {
        (true, Some(search)) => {
            sqlx::query_as!(
                Customer,
                r#"
                SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                FROM customers 
                WHERE company_id = $1 
                  AND (customer_name ILIKE $2 OR customer_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                ORDER BY customer_name
                "#,
                company_id,
                format!("%{}%", search)
            )
            .fetch_all(&state.db)
            .await
        }
        (false, Some(search)) => {
            sqlx::query_as!(
                Customer,
                r#"
                SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                FROM customers 
                WHERE company_id = $1 AND is_active = true
                  AND (customer_name ILIKE $2 OR customer_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                ORDER BY customer_name
                "#,
                company_id,
                format!("%{}%", search)
            )
            .fetch_all(&state.db)
            .await
        }
        (true, None) => {
            sqlx::query_as!(
                Customer,
                r#"
                SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                FROM customers 
                WHERE company_id = $1
                ORDER BY customer_name
                "#,
                company_id
            )
            .fetch_all(&state.db)
            .await
        }
        (false, None) => {
            sqlx::query_as!(
                Customer,
                r#"
                SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                FROM customers 
                WHERE company_id = $1 AND is_active = true
                ORDER BY customer_name
                "#,
                company_id
            )
            .fetch_all(&state.db)
            .await
        }
    };

    let customers = customers.map_err(|e| {
        error!("Failed to fetch customers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(customers))
}

async fn update_customer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(customer_id): Path<Uuid>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> Result<Json<Customer>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let customer = sqlx::query_as!(
        Customer,
        r#"
        UPDATE customers 
        SET customer_name = $1, npwp = $2, address = $3, phone = $4, email = $5, 
            credit_limit = $6, payment_terms = $7, is_active = $8, updated_at = NOW()
        WHERE id = $9 AND company_id = $10
        RETURNING id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
        "#,
        payload.customer_name,
        payload.npwp,
        payload.address,
        payload.phone,
        payload.email,
        payload.credit_limit,
        payload.payment_terms,
        payload.is_active,
        customer_id,
        company_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Log audit trail
    state.log_customer_activity(&mut tx, customer_id, "UPDATE", user_id).await
        .map_err(|e| {
            error!("Failed to log customer activity: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Updated customer {} for company {}", customer.customer_code, company_id);

    Ok(Json(customer))
}

async fn create_customer_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateCustomerInvoiceRequest>,
) -> Result<Json<CustomerInvoice>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let _user_id = extract_user_id(&headers)?;
    payload.company_id = company_id;
    
    let customer = sqlx::query!(
        "SELECT payment_terms, customer_name, credit_limit FROM customers WHERE id = $1 AND company_id = $2 AND is_active = true", 
        payload.customer_id,
        company_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::BAD_REQUEST)?;

    let total_amount = payload.subtotal + payload.tax_amount;

    // Check credit limit
    let credit_available = state.check_credit_limit(payload.customer_id, total_amount).await
        .map_err(|e| {
            error!("Failed to check credit limit: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !credit_available {
        warn!("Credit limit exceeded for customer {}", payload.customer_id);
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let due_date = payload.invoice_date + chrono::Duration::days(customer.payment_terms as i64);
    let invoice_id = Uuid::new_v4();

    // Check for duplicate invoice number
    let existing_invoice = sqlx::query!(
        "SELECT id FROM customer_invoices WHERE company_id = $1 AND customer_id = $2 AND invoice_number = $3",
        company_id,
        payload.customer_id,
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
        INSERT INTO customer_invoices 
        (id, company_id, customer_id, invoice_number, invoice_date, due_date, subtotal, tax_amount, total_amount, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
        RETURNING id, company_id, customer_id, invoice_number, invoice_date, due_date,
                  subtotal, tax_amount, total_amount, paid_amount,
                  status as "status_str", description, journal_entry_id, created_at, updated_at
        "#,
        invoice_id,
        payload.company_id,
        payload.customer_id,
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
        error!("Failed to create customer invoice: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let status = invoice.status_str.as_deref()
        .and_then(|s| s.parse::<InvoiceStatus>().ok())
        .unwrap_or(InvoiceStatus::Draft);

    let customer_invoice = CustomerInvoice {
        id: invoice.id,
        company_id: invoice.company_id,
        customer_id: invoice.customer_id,
        customer_name: Some(customer.customer_name),
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

    info!("Created customer invoice {} for company {}", payload.invoice_number, company_id);

    Ok(Json(customer_invoice))
}

async fn get_customer_invoices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<CustomerInvoice>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let status_filter = params.get("status");
    let customer_id_filter = params.get("customer_id")
        .and_then(|id| Uuid::parse_str(id).ok());

    let limit: i64 = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50)
        .min(200);

    let offset: i64 = params.get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    let invoices_data = match (status_filter, customer_id_filter) {
        (Some(status), Some(customer_id)) => {
            sqlx::query!(
                r#"
                SELECT ci.id, ci.company_id, ci.customer_id, c.customer_name, ci.invoice_number, 
                       ci.invoice_date, ci.due_date, ci.subtotal, ci.tax_amount, ci.total_amount, 
                       ci.paid_amount, ci.status as "status_str", ci.description, 
                       ci.journal_entry_id, ci.created_at, ci.updated_at
                FROM customer_invoices ci
                LEFT JOIN customers c ON ci.customer_id = c.id
                WHERE ci.company_id = $1 AND ci.status = $2::invoice_status AND ci.customer_id = $3
                ORDER BY ci.invoice_date DESC, ci.created_at DESC
                LIMIT $4 OFFSET $5
                "#,
                company_id,
                status,
                customer_id,
                limit,
                offset
            )
            .fetch_all(&state.db)
            .await
        }
        (Some(status), None) => {
            sqlx::query!(
                r#"
                SELECT ci.id, ci.company_id, ci.customer_id, c.customer_name, ci.invoice_number, 
                       ci.invoice_date, ci.due_date, ci.subtotal, ci.tax_amount, ci.total_amount, 
                       ci.paid_amount, ci.status as "status_str", ci.description, 
                       ci.journal_entry_id, ci.created_at, ci.updated_at
                FROM customer_invoices ci
                LEFT JOIN customers c ON ci.customer_id = c.id
                WHERE ci.company_id = $1 AND ci.status = $2::invoice_status
                ORDER BY ci.invoice_date DESC, ci.created_at DESC
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
        (None, Some(customer_id)) => {
            sqlx::query!(
                r#"
                SELECT ci.id, ci.company_id, ci.customer_id, c.customer_name, ci.invoice_number, 
                       ci.invoice_date, ci.due_date, ci.subtotal, ci.tax_amount, ci.total_amount, 
                       ci.paid_amount, ci.status as "status_str", ci.description, 
                       ci.journal_entry_id, ci.created_at, ci.updated_at
                FROM customer_invoices ci
                LEFT JOIN customers c ON ci.customer_id = c.id
                WHERE ci.company_id = $1 AND ci.customer_id = $2
                ORDER BY ci.invoice_date DESC, ci.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                company_id,
                customer_id,
                limit,
                offset
            )
            .fetch_all(&state.db)
            .await
        }
        (None, None) => {
            sqlx::query!(
                r#"
                SELECT ci.id, ci.company_id, ci.customer_id, c.customer_name, ci.invoice_number, 
                       ci.invoice_date, ci.due_date, ci.subtotal, ci.tax_amount, ci.total_amount, 
                       ci.paid_amount, ci.status as "status_str", ci.description, 
                       ci.journal_entry_id, ci.created_at, ci.updated_at
                FROM customer_invoices ci
                LEFT JOIN customers c ON ci.customer_id = c.id
                WHERE ci.company_id = $1
                ORDER BY ci.invoice_date DESC, ci.created_at DESC
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
        error!("Failed to fetch customer invoices: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let invoices: Vec<CustomerInvoice> = invoices_data
        .into_iter()
        .map(|row| {
            let status = row.status_str.as_deref()
                .and_then(|s| s.parse::<InvoiceStatus>().ok())
                .unwrap_or(InvoiceStatus::Draft);

            CustomerInvoice {
                id: row.id,
                company_id: row.company_id,
                customer_id: row.customer_id,
                customer_name: row.customer_name,
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

async fn receive_payment(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<PaymentRequest>,
) -> Result<Json<CustomerInvoice>, StatusCode> {
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
        "SELECT total_amount, paid_amount FROM customer_invoices WHERE id = $1 AND company_id = $2",
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
        UPDATE customer_invoices 
        SET paid_amount = $1,
            status = $2::invoice_status,
            updated_at = NOW()
        WHERE id = $3 AND company_id = $4
        RETURNING id, company_id, customer_id, invoice_number, invoice_date, due_date,
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

    // Get customer name
    let customer_name = sqlx::query_scalar!(
        "SELECT customer_name FROM customers WHERE id = $1",
        invoice_data.customer_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to fetch customer name: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let status = invoice_data.status_str.as_deref()
        .and_then(|s| s.parse::<InvoiceStatus>().ok())
        .unwrap_or(InvoiceStatus::Draft);

    let invoice = CustomerInvoice {
        id: invoice_data.id,
        company_id: invoice_data.company_id,
        customer_id: invoice_data.customer_id,
        customer_name,
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

async fn get_customer_aging_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<CustomerAgingReport>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let as_of_date = params.get("as_of_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().naive_local().date());

    let aging_data = sqlx::query!(
        r#"  
        SELECT 
            c.id as customer_id,
            c.customer_name,
            c.credit_limit,
            ci.id as invoice_id,
            ci.invoice_number,
            ci.invoice_date,
            ci.due_date,
            ci.total_amount - ci.paid_amount as outstanding_amount,
            $2 - ci.due_date as days_overdue
        FROM customer_invoices ci
        JOIN customers c ON ci.customer_id = c.id
        WHERE ci.company_id = $1 
              AND ci.status != 'PAID'
              AND ci.total_amount > ci.paid_amount
        ORDER BY c.customer_name, ci.due_date
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

    let mut customer_details: std::collections::HashMap<Uuid, CustomerAgingDetail> = std::collections::HashMap::new();
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

        // Update customer detail
        let customer_detail = customer_details.entry(row.customer_id).or_insert_with(|| {
            CustomerAgingDetail {
                customer_id: row.customer_id,
                customer_name: row.customer_name.clone(),
                credit_limit: row.credit_limit,
                current: Decimal::ZERO,
                days_31_60: Decimal::ZERO,
                days_61_90: Decimal::ZERO,
                over_90_days: Decimal::ZERO,
                total_outstanding: Decimal::ZERO,
                credit_utilization: 0.0,
                invoices: Vec::new(),
            }
        });

        customer_detail.total_outstanding += outstanding;
        match days_overdue {
            d if d <= 30 => customer_detail.current += outstanding,
            d if d <= 60 => customer_detail.days_31_60 += outstanding,
            d if d <= 90 => customer_detail.days_61_90 += outstanding,
            _ => customer_detail.over_90_days += outstanding,
        }

        customer_detail.invoices.push(InvoiceAgingItem {
            invoice_id: row.invoice_id,
            invoice_number: row.invoice_number,
            invoice_date: row.invoice_date,
            due_date: row.due_date,
            days_overdue,
            outstanding_amount: outstanding,
        });
    }

    // Calculate credit utilization for each customer
    for customer in customer_details.values_mut() {
        if customer.credit_limit > Decimal::ZERO {
            customer.credit_utilization = (customer.total_outstanding / customer.credit_limit).to_string().parse().unwrap_or(0.0) * 100.0;
        }
    }

    let report = CustomerAgingReport {
        company_id,
        report_date: as_of_date,
        summary,
        customer_details: customer_details.into_values().collect(),
        generated_at: chrono::Utc::now(),
    };

    info!("Generated customer aging report for company {} with {} customers", company_id, report.customer_details.len());

    Ok(Json(report))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,accounts_receivable=debug")
        .init();

    info!("Starting Accounts Receivable Service...");

    // Use the dedicated database pool creation function
    let pool = create_database_pool().await?;

    let app_state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/customers", post(create_customer))
        .route("/customers", get(get_customers))
        .route("/customers/:id", put(update_customer))
        .route("/invoices", post(create_customer_invoice))
        .route("/invoices", get(get_customer_invoices))
        .route("/invoices/:id/payment", put(receive_payment))
        .route("/aging-report", get(get_customer_aging_report))
        .with_state(app_state);

    let bind_addr = std::env::var("ACCOUNTS_RECEIVABLE_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3007".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Accounts Receivable service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}