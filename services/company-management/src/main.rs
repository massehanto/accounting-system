mod handlers;
mod models;
mod services;
mod utils;

use axum::{routing::{get, post, put}, Router};
use handlers::*;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    company_service: services::CompanyService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,company_management=debug")
        .init();

    info!("Starting Company Management Service...");

    let pool = database::create_database_pool("company-management").await?;
    let company_service = services::CompanyService::new(pool.clone());

    let app_state = Arc::new(AppState { 
        db: pool,
        company_service,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/companies", post(create_company))
        .route("/companies", get(get_companies))
        .route("/companies/:id", get(get_company))
        .route("/companies/:id", put(update_company))
        .route("/companies/:id/settings", get(get_company_settings))
        .route("/companies/:id/settings", put(update_company_settings))
        .with_state(app_state);

    let bind_addr = std::env::var("COMPANY_MANAGEMENT_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3002".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Company Management service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}