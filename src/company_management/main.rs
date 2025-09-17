// src/company_management/main.rs
use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::{Json, IntoResponse},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{sync::Arc, env};
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
    let database_url = env::var("COMPANY_MANAGEMENT_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("COMPANY_MANAGEMENT_DATABASE_URL must be set"))?;

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

    info!("Connected to company database");
    Ok(pool)
}

async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

#[derive(Debug, Serialize, Deserialize)]
struct Company {
    id: Uuid,
    name: String,
    npwp: String,
    address: String,
    phone: Option<String>,
    email: Option<String>,
    business_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateCompanyRequest {
    name: String,
    npwp: String,
    address: String,
    phone: Option<String>,
    email: Option<String>,
    business_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateCompanyRequest {
    name: String,
    address: String,
    phone: Option<String>,
    email: Option<String>,
    business_type: Option<String>,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

async fn create_company(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateCompanyRequest>,
) -> Result<Json<Company>, StatusCode> {
    let _user_id = extract_user_id(&headers)?;
    
    let company_id = Uuid::new_v4();
    
    let company = sqlx::query_as!(
        Company,
        r#"
        INSERT INTO companies (id, name, npwp, address, phone, email, business_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, name, npwp, address, phone, email, business_type
        "#,
        company_id,
        payload.name,
        payload.npwp,
        payload.address,
        payload.phone,
        payload.email,
        payload.business_type
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create company: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Company created: {} ({})", company.name, company.id);
    Ok(Json(company))
}

async fn get_companies(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Company>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    
    let companies = sqlx::query_as!(
        Company,
        "SELECT id, name, npwp, address, phone, email, business_type FROM companies WHERE id = $1",
        company_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch companies: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(companies))
}

async fn get_company(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Company>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    
    if id != company_id {
        warn!("User attempted to access different company: {} vs {}", company_id, id);
        return Err(StatusCode::FORBIDDEN);
    }
    
    let company = sqlx::query_as!(
        Company,
        "SELECT id, name, npwp, address, phone, email, business_type FROM companies WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch company: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(company))
}

async fn update_company(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCompanyRequest>,
) -> Result<Json<Company>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    
    if id != company_id {
        warn!("User attempted to update different company: {} vs {}", company_id, id);
        return Err(StatusCode::FORBIDDEN);
    }
    
    let company = sqlx::query_as!(
        Company,
        r#"
        UPDATE companies 
        SET name = $1, address = $2, phone = $3, email = $4, business_type = $5, updated_at = NOW()
        WHERE id = $6
        RETURNING id, name, npwp, address, phone, email, business_type
        "#,
        payload.name,
        payload.address,
        payload.phone,
        payload.email,
        payload.business_type,
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to update company: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    info!("Company updated: {} ({})", company.name, company.id);
    Ok(Json(company))
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
        "service": "company-management-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,company_management=debug")
        .init();

    info!("Starting Company Management Service...");

    let pool = create_database_pool().await?;
    let app_state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/companies", post(create_company))
        .route("/companies", get(get_companies))
        .route("/companies/:id", get(get_company))
        .route("/companies/:id", put(update_company))
        .with_state(app_state);

    let bind_addr = std::env::var("COMPANY_MANAGEMENT_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3002".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Company Management service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}