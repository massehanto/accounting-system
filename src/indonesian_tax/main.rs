// src/indonesian_tax/main.rs
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
use sqlx::{PgPool, Type, postgres::PgPoolOptions};
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

// Database connection function specific to Indonesian tax service
async fn create_tax_database_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("INDONESIAN_TAX_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("INDONESIAN_TAX_DATABASE_URL must be set"))?;

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

    info!("Connected to Indonesian tax database: {}", database_url.split('@').last().unwrap_or("unknown"));
    Ok(pool)
}

async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "tax_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaxType {
    Ppn,    // Value Added Tax
    Pph21,  // Income Tax Article 21 (Employee)
    Pph22,  // Income Tax Article 22 (Import/Purchase)
    Pph23,  // Income Tax Article 23 (Services)
    Pph25,  // Income Tax Article 25 (Monthly Installment)
    Pph29,  // Income Tax Article 29 (Annual)
    Pbb,    // Property Tax
}

impl std::str::FromStr for TaxType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PPN" => Ok(TaxType::Ppn),
            "PPH21" => Ok(TaxType::Pph21),
            "PPH22" => Ok(TaxType::Pph22),
            "PPH23" => Ok(TaxType::Pph23),
            "PPH25" => Ok(TaxType::Pph25),
            "PPH29" => Ok(TaxType::Pph29),
            "PBB" => Ok(TaxType::Pbb),
            _ => Err(format!("Invalid tax type: {}", s))
        }
    }
}

impl std::fmt::Display for TaxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaxType::Ppn => write!(f, "PPN"),
            TaxType::Pph21 => write!(f, "PPH21"),
            TaxType::Pph22 => write!(f, "PPH22"),
            TaxType::Pph23 => write!(f, "PPH23"),
            TaxType::Pph25 => write!(f, "PPH25"),
            TaxType::Pph29 => write!(f, "PPH29"),
            TaxType::Pbb => write!(f, "PBB"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TaxConfiguration {
    id: Uuid,
    company_id: Uuid,
    tax_type: TaxType,
    tax_rate: Decimal,
    is_active: bool,
    effective_date: NaiveDate,
    end_date: Option<NaiveDate>,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaxTransaction {
    id: Uuid,
    company_id: Uuid,
    tax_type: TaxType,
    transaction_date: NaiveDate,
    tax_period: NaiveDate,
    tax_base_amount: Decimal,
    tax_amount: Decimal,
    tax_invoice_number: Option<String>,
    vendor_npwp: Option<String>,
    vendor_name: Option<String>,
    description: Option<String>,
    journal_entry_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTaxConfigRequest {
    company_id: Uuid,
    tax_type: TaxType,
    tax_rate: Decimal,
    effective_date: NaiveDate,
    end_date: Option<NaiveDate>,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTaxTransactionRequest {
    company_id: Uuid,
    tax_type: TaxType,
    transaction_date: NaiveDate,
    tax_period: NaiveDate,
    tax_base_amount: Decimal,
    tax_invoice_number: Option<String>,
    vendor_npwp: Option<String>,
    vendor_name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaxReport {
    company_id: Uuid,
    tax_type: TaxType,
    period_start: NaiveDate,
    period_end: NaiveDate,
    total_tax_base: Decimal,
    total_tax_amount: Decimal,
    transaction_count: usize,
    transactions: Vec<TaxTransactionSummary>,
    generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaxTransactionSummary {
    id: Uuid,
    transaction_date: NaiveDate,
    tax_base_amount: Decimal,
    tax_amount: Decimal,
    tax_invoice_number: Option<String>,
    vendor_name: Option<String>,
    description: Option<String>,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

pub struct TaxCalculator;

impl TaxCalculator {
    pub fn calculate_ppn(&self, base_amount: Decimal, rate: Decimal) -> Decimal {
        base_amount * rate / Decimal::new(100, 0)
    }
    
    pub fn calculate_pph21(&self, gross_salary: Decimal, ptkp: Decimal) -> Decimal {
        let taxable_income = (gross_salary - ptkp).max(Decimal::ZERO);
        if taxable_income <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        self.apply_progressive_rates(taxable_income)
    }
    
    pub fn calculate_pph23(&self, service_amount: Decimal, rate: Decimal) -> Decimal {
        service_amount * rate / Decimal::new(100, 0)
    }
    
    fn apply_progressive_rates(&self, income: Decimal) -> Decimal {
        let mut tax = Decimal::ZERO;
        let mut remaining = income;
        
        // 2024 Indonesian tax brackets
        // 5% bracket (0 - 60M)
        if remaining > Decimal::ZERO {
            let bracket_amount = remaining.min(Decimal::new(60_000_000, 0));
            tax += bracket_amount * Decimal::new(5, 0) / Decimal::new(100, 0);
            remaining -= bracket_amount;
        }
        
        // 15% bracket (60M - 250M)
        if remaining > Decimal::ZERO {
            let bracket_amount = remaining.min(Decimal::new(190_000_000, 0));
            tax += bracket_amount * Decimal::new(15, 0) / Decimal::new(100, 0);
            remaining -= bracket_amount;
        }
        
        // 25% bracket (250M - 500M)
        if remaining > Decimal::ZERO {
            let bracket_amount = remaining.min(Decimal::new(250_000_000, 0));
            tax += bracket_amount * Decimal::new(25, 0) / Decimal::new(100, 0);
            remaining -= bracket_amount;
        }
        
        // 30% bracket (above 500M)
        if remaining > Decimal::ZERO {
            tax += remaining * Decimal::new(30, 0) / Decimal::new(100, 0);
        }
        
        tax
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
        "service": "indonesian-tax-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    })))
}

async fn create_tax_configuration(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateTaxConfigRequest>,
) -> Result<Json<TaxConfiguration>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let _user_id = extract_user_id(&headers)?; // For audit logging
    payload.company_id = company_id;
    
    let config_id = Uuid::new_v4();
    
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Deactivate previous configurations for the same tax type
    sqlx::query!(
        "UPDATE tax_configurations SET is_active = false, end_date = CURRENT_DATE WHERE company_id = $1 AND tax_type = $2 AND is_active = true",
        payload.company_id,
        payload.tax_type as TaxType
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to deactivate old tax configurations: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let config = sqlx::query_as!(
        TaxConfiguration,
        r#"
        INSERT INTO tax_configurations (id, company_id, tax_type, tax_rate, effective_date, end_date, description, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
        RETURNING id, company_id, tax_type as "tax_type: TaxType", tax_rate, is_active, 
                  effective_date, end_date, description, created_at, updated_at
        "#,
        config_id,
        payload.company_id,
        payload.tax_type as TaxType,
        payload.tax_rate,
        payload.effective_date,
        payload.end_date,
        payload.description
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to create tax configuration: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created tax configuration for company {} with type {:?}", company_id, payload.tax_type);

    Ok(Json(config))
}

async fn get_tax_configurations(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TaxConfiguration>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let include_inactive = params.get("include_inactive")
        .map(|v| v == "true")
        .unwrap_or(false);

    let configs = if include_inactive {
        sqlx::query_as!(
            TaxConfiguration,
            r#"
            SELECT id, company_id, tax_type as "tax_type: TaxType", tax_rate, is_active, 
                   effective_date, end_date, description, created_at, updated_at
            FROM tax_configurations 
            WHERE company_id = $1
            ORDER BY tax_type, effective_date DESC
            "#,
            company_id
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as!(
            TaxConfiguration,
            r#"
            SELECT id, company_id, tax_type as "tax_type: TaxType", tax_rate, is_active, 
                   effective_date, end_date, description, created_at, updated_at
            FROM tax_configurations 
            WHERE company_id = $1 AND is_active = true
            ORDER BY tax_type, effective_date DESC
            "#,
            company_id
        )
        .fetch_all(&state.db)
        .await
    };

    let configs = configs.map_err(|e| {
        error!("Failed to fetch tax configurations: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(configs))
}

async fn create_tax_transaction(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateTaxTransactionRequest>,
) -> Result<Json<TaxTransaction>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    let _user_id = extract_user_id(&headers)?;
    payload.company_id = company_id;
    
    // Get active tax configuration
    let tax_config = sqlx::query!(
        r#"
        SELECT tax_rate FROM tax_configurations 
        WHERE company_id = $1 AND tax_type = $2 AND is_active = true
        AND effective_date <= $3 AND (end_date IS NULL OR end_date > $3)
        ORDER BY effective_date DESC
        LIMIT 1
        "#,
        payload.company_id,
        payload.tax_type as TaxType,
        payload.transaction_date
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch tax configuration: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        warn!("No active tax configuration found for type {:?} on date {}", payload.tax_type, payload.transaction_date);
        StatusCode::BAD_REQUEST
    })?;

    let calculator = TaxCalculator;
    let tax_amount = match payload.tax_type {
        TaxType::Ppn => calculator.calculate_ppn(payload.tax_base_amount, tax_config.tax_rate),
        TaxType::Pph23 => calculator.calculate_pph23(payload.tax_base_amount, tax_config.tax_rate),
        _ => payload.tax_base_amount * tax_config.tax_rate / Decimal::new(100, 0),
    };

    let transaction_id = Uuid::new_v4();

    let transaction = sqlx::query_as!(
        TaxTransaction,
        r#"
        INSERT INTO tax_transactions 
        (id, company_id, tax_type, transaction_date, tax_period, tax_base_amount, tax_amount, 
         tax_invoice_number, vendor_npwp, vendor_name, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
        RETURNING id, company_id, tax_type as "tax_type: TaxType", transaction_date, tax_period,
                  tax_base_amount, tax_amount, tax_invoice_number, vendor_npwp, vendor_name,
                  description, journal_entry_id, created_at, updated_at
        "#,
        transaction_id,
        payload.company_id,
        payload.tax_type as TaxType,
        payload.transaction_date,
        payload.tax_period,
        payload.tax_base_amount,
        tax_amount,
        payload.tax_invoice_number,
        payload.vendor_npwp,
        payload.vendor_name,
        payload.description
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create tax transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created tax transaction {} for company {}", transaction_id, company_id);

    Ok(Json(transaction))
}

async fn get_tax_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<TaxReport>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    
    let tax_type_str = params.get("tax_type").ok_or_else(|| {
        warn!("Missing tax_type parameter");
        StatusCode::BAD_REQUEST
    })?;
    
    let tax_type: TaxType = tax_type_str.parse()
        .map_err(|e: String| {
            warn!("Invalid tax type provided: {} - {}", tax_type_str, e);
            StatusCode::BAD_REQUEST
        })?;

    let period_start = params.get("period_start")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| {
            warn!("Invalid or missing period_start parameter");
            StatusCode::BAD_REQUEST
        })?;
    
    let period_end = params.get("period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| {
            warn!("Invalid or missing period_end parameter");
            StatusCode::BAD_REQUEST
        })?;

    let transactions = sqlx::query!(
        r#"
        SELECT id, transaction_date, tax_base_amount, tax_amount, 
               tax_invoice_number, vendor_name, description
        FROM tax_transactions
        WHERE company_id = $1 AND tax_type = $2 
              AND transaction_date >= $3 AND transaction_date <= $4
        ORDER BY transaction_date DESC
        "#,
        company_id,
        tax_type as TaxType,
        period_start,
        period_end
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch tax transactions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total_tax_base: Decimal = transactions.iter().map(|t| t.tax_base_amount).sum();
    let total_tax_amount: Decimal = transactions.iter().map(|t| t.tax_amount).sum();

    let transaction_summaries: Vec<TaxTransactionSummary> = transactions
        .into_iter()
        .map(|t| TaxTransactionSummary {
            id: t.id,
            transaction_date: t.transaction_date,
            tax_base_amount: t.tax_base_amount,
            tax_amount: t.tax_amount,
            tax_invoice_number: t.tax_invoice_number,
            vendor_name: t.vendor_name,
            description: t.description,
        })
        .collect();

    let report = TaxReport {
        company_id,
        tax_type,
        period_start,
        period_end,
        total_tax_base,
        total_tax_amount,
        transaction_count: transaction_summaries.len(),
        transactions: transaction_summaries,
        generated_at: chrono::Utc::now(),
    };

    info!("Generated tax report for company {} with {} transactions", company_id, report.transaction_count);

    Ok(Json(report))
}

async fn get_tax_calculations(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _company_id = extract_company_id(&headers)?;
    
    let tax_type_str = params.get("tax_type").ok_or_else(|| {
        warn!("Missing tax_type parameter");
        StatusCode::BAD_REQUEST
    })?;
    
    let tax_type: TaxType = tax_type_str.parse()
        .map_err(|e: String| {
            warn!("Invalid tax type provided: {} - {}", tax_type_str, e);
            StatusCode::BAD_REQUEST
        })?;

    let base_amount = params.get("base_amount")
        .and_then(|a| a.parse::<Decimal>().ok())
        .ok_or_else(|| {
            warn!("Invalid or missing base_amount parameter");
            StatusCode::BAD_REQUEST
        })?;

    let tax_rate = params.get("tax_rate")
        .and_then(|r| r.parse::<Decimal>().ok())
        .unwrap_or_else(|| {
            // Use default rates for Indonesian taxes
            match tax_type {
                TaxType::Ppn => Decimal::new(11, 0), // 11% PPN
                TaxType::Pph21 => Decimal::new(5, 0), // Progressive, but simplified
                TaxType::Pph22 => Decimal::new(15, 1), // 1.5%
                TaxType::Pph23 => Decimal::new(2, 0), // 2%
                TaxType::Pph25 => Decimal::new(1, 0), // 1%
                TaxType::Pph29 => Decimal::new(25, 0), // 25%
                TaxType::Pbb => Decimal::new(5, 1), // 0.5%
            }
        });

    let calculator = TaxCalculator;
    
    let tax_amount = match tax_type {
        TaxType::Ppn => calculator.calculate_ppn(base_amount, tax_rate),
        TaxType::Pph21 => {
            // For PPh21, base_amount is gross salary, we need PTKP
            let ptkp = params.get("ptkp")
                .and_then(|p| p.parse::<Decimal>().ok())
                .unwrap_or(Decimal::new(54_000_000, 0)); // 2024 single PTKP
            calculator.calculate_pph21(base_amount, ptkp)
        },
        TaxType::Pph23 => calculator.calculate_pph23(base_amount, tax_rate),
        _ => base_amount * tax_rate / Decimal::new(100, 0),
    };

    let calculation_result = serde_json::json!({
        "tax_type": tax_type.to_string(),
        "base_amount": base_amount,
        "tax_rate": tax_rate,
        "tax_amount": tax_amount,
        "total_amount": base_amount + tax_amount,
        "calculation_method": match tax_type {
            TaxType::Ppn => "Standard rate multiplication",
            TaxType::Pph21 => "Progressive rates after PTKP deduction",
            TaxType::Pph23 => "Withholding tax calculation",
            _ => "Standard rate multiplication"
        },
        "calculated_at": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(calculation_result))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,indonesian_tax=debug")
        .init();

    info!("Starting Indonesian Tax Service...");

    // Use the local database pool creation
    let pool = create_tax_database_pool().await?;

    let app_state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/tax-configurations", post(create_tax_configuration))
        .route("/tax-configurations", get(get_tax_configurations))
        .route("/tax-transactions", post(create_tax_transaction))
        .route("/tax-report", get(get_tax_report))
        .route("/tax-calculations", get(get_tax_calculations))
        .with_state(app_state);

    let bind_addr = env::var("INDONESIAN_TAX_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3005".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Indonesian Tax service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}