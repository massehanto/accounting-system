// src/chart_of_accounts/main.rs
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::{Json, IntoResponse},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type, postgres::PgPoolOptions};
use std::{sync::Arc, str::FromStr, env};
use uuid::Uuid;
use tracing::{info, warn, error};

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

async fn create_database_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("CHART_OF_ACCOUNTS_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("CHART_OF_ACCOUNTS_DATABASE_URL must be set"))?;

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

    info!("Connected to chart of accounts database");
    Ok(pool)
}

async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "account_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl FromStr for AccountType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ASSET" => Ok(AccountType::Asset),
            "LIABILITY" => Ok(AccountType::Liability),
            "EQUITY" => Ok(AccountType::Equity),
            "REVENUE" => Ok(AccountType::Revenue),
            "EXPENSE" => Ok(AccountType::Expense),
            _ => Err(format!("Invalid account type: {}", s))
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Asset => write!(f, "ASSET"),
            AccountType::Liability => write!(f, "LIABILITY"),
            AccountType::Equity => write!(f, "EQUITY"),
            AccountType::Revenue => write!(f, "REVENUE"),
            AccountType::Expense => write!(f, "EXPENSE"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "account_subtype", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountSubtype {
    CurrentAsset,
    FixedAsset,
    OtherAsset,
    CurrentLiability,
    LongTermLiability,
    OwnerEquity,
    RetainedEarnings,
    OperatingRevenue,
    NonOperatingRevenue,
    CostOfGoodsSold,
    OperatingExpense,
    NonOperatingExpense,
}

impl FromStr for AccountSubtype {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CURRENT_ASSET" => Ok(AccountSubtype::CurrentAsset),
            "FIXED_ASSET" => Ok(AccountSubtype::FixedAsset),
            "OTHER_ASSET" => Ok(AccountSubtype::OtherAsset),
            "CURRENT_LIABILITY" => Ok(AccountSubtype::CurrentLiability),
            "LONG_TERM_LIABILITY" => Ok(AccountSubtype::LongTermLiability),
            "OWNER_EQUITY" => Ok(AccountSubtype::OwnerEquity),
            "RETAINED_EARNINGS" => Ok(AccountSubtype::RetainedEarnings),
            "OPERATING_REVENUE" => Ok(AccountSubtype::OperatingRevenue),
            "NON_OPERATING_REVENUE" => Ok(AccountSubtype::NonOperatingRevenue),
            "COST_OF_GOODS_SOLD" => Ok(AccountSubtype::CostOfGoodsSold),
            "OPERATING_EXPENSE" => Ok(AccountSubtype::OperatingExpense),
            "NON_OPERATING_EXPENSE" => Ok(AccountSubtype::NonOperatingExpense),
            _ => Err(format!("Invalid account subtype: {}", s))
        }
    }
}

impl std::fmt::Display for AccountSubtype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountSubtype::CurrentAsset => write!(f, "CURRENT_ASSET"),
            AccountSubtype::FixedAsset => write!(f, "FIXED_ASSET"),
            AccountSubtype::OtherAsset => write!(f, "OTHER_ASSET"),
            AccountSubtype::CurrentLiability => write!(f, "CURRENT_LIABILITY"),
            AccountSubtype::LongTermLiability => write!(f, "LONG_TERM_LIABILITY"),
            AccountSubtype::OwnerEquity => write!(f, "OWNER_EQUITY"),
            AccountSubtype::RetainedEarnings => write!(f, "RETAINED_EARNINGS"),
            AccountSubtype::OperatingRevenue => write!(f, "OPERATING_REVENUE"),
            AccountSubtype::NonOperatingRevenue => write!(f, "NON_OPERATING_REVENUE"),
            AccountSubtype::CostOfGoodsSold => write!(f, "COST_OF_GOODS_SOLD"),
            AccountSubtype::OperatingExpense => write!(f, "OPERATING_EXPENSE"),
            AccountSubtype::NonOperatingExpense => write!(f, "NON_OPERATING_EXPENSE"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    id: Uuid,
    company_id: Uuid,
    account_code: String,
    account_name: String,
    account_type: AccountType,
    account_subtype: Option<AccountSubtype>,
    parent_account_id: Option<Uuid>,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateAccountRequest {
    company_id: Uuid,
    account_code: String,
    account_name: String,
    account_type: AccountType,
    account_subtype: Option<AccountSubtype>,
    parent_account_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateAccountRequest {
    account_name: String,
    account_type: AccountType,
    account_subtype: Option<AccountSubtype>,
    parent_account_id: Option<Uuid>,
    is_active: bool,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

async fn create_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateAccountRequest>,
) -> Result<Json<Account>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    payload.company_id = company_id;
    
    let account_id = Uuid::new_v4();
    
    let account = sqlx::query_as!(
        Account,
        r#"
        INSERT INTO accounts (id, company_id, account_code, account_name, account_type, account_subtype, parent_account_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, company_id, account_code, account_name, 
                  account_type as "account_type: AccountType", 
                  account_subtype as "account_subtype: Option<AccountSubtype>", 
                  parent_account_id, is_active
        "#,
        account_id,
        payload.company_id,
        payload.account_code,
        payload.account_name,
        payload.account_type as AccountType,
        payload.account_subtype as Option<AccountSubtype>,
        payload.parent_account_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create account: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Account created: {} - {} ({})", account.account_code, account.account_name, account.id);
    Ok(Json(account))
}

async fn get_accounts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<Account>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let accounts = sqlx::query_as!(
        Account,
        r#"
        SELECT id, company_id, account_code, account_name,
               account_type as "account_type: AccountType",
               account_subtype as "account_subtype: Option<AccountSubtype>",
               parent_account_id, is_active
        FROM accounts 
        WHERE company_id = $1 
        ORDER BY account_code
        "#,
        company_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch accounts: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(accounts))
}

async fn get_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Account>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let account = sqlx::query_as!(
        Account,
        r#"
        SELECT id, company_id, account_code, account_name,
               account_type as "account_type: AccountType",
               account_subtype as "account_subtype: Option<AccountSubtype>",
               parent_account_id, is_active
        FROM accounts 
        WHERE id = $1 AND company_id = $2
        "#,
        id,
        company_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch account: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(account))
}

async fn update_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAccountRequest>,
) -> Result<Json<Account>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let account = sqlx::query_as!(
        Account,
        r#"
        UPDATE accounts 
        SET account_name = $1, account_type = $2, account_subtype = $3, 
            parent_account_id = $4, is_active = $5, updated_at = NOW()
        WHERE id = $6 AND company_id = $7
        RETURNING id, company_id, account_code, account_name,
                  account_type as "account_type: AccountType",
                  account_subtype as "account_subtype: Option<AccountSubtype>",
                  parent_account_id, is_active
        "#,
        payload.account_name,
        payload.account_type as AccountType,
        payload.account_subtype as Option<AccountSubtype>,
        payload.parent_account_id,
        payload.is_active,
        id,
        company_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to update account: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    info!("Account updated: {} - {} ({})", account.account_code, account.account_name, account.id);
    Ok(Json(account))
}

async fn delete_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let result = sqlx::query!(
        "DELETE FROM accounts WHERE id = $1 AND company_id = $2 AND is_system = false", 
        id, 
        company_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to delete account: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    info!("Account deleted: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_healthy = check_database_health(&state.db).await;
    
    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy { 
        StatusCode::OK 
    } else { 
        StatusCode::SERVICE_UNAVAILABLE 
    };
    
    (status_code, Json(serde_json::json!({
        "service": "chart-of-accounts-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,chart_of_accounts=debug")
        .init();

    info!("Starting Chart of Accounts Service...");

    let pool = create_database_pool().await?;
    let app_state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/accounts", post(create_account))
        .route("/accounts", get(get_accounts))
        .route("/accounts/:id", get(get_account))
        .route("/accounts/:id", put(update_account))
        .route("/accounts/:id", delete(delete_account))
        .with_state(app_state);

    let bind_addr = std::env::var("CHART_OF_ACCOUNTS_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3003".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Chart of Accounts service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}