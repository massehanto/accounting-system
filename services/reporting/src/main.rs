mod handlers;
mod models;
mod services;
mod utils;

use axum::{routing::get, Router};
use handlers::*;
use std::sync::Arc;
use tracing::info;
use common::ServiceResult;

#[derive(Clone)]
pub struct AppState {
    http_client: reqwest::Client,
    service_registry: services::ServiceRegistry,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,reporting_service=debug")
        .init();

    info!("Starting Reporting Service...");

    let service_registry = services::ServiceRegistry::new();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let app_state = Arc::new(AppState {
        http_client,
        service_registry,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/balance-sheet", get(generate_balance_sheet))
        .route("/income-statement", get(generate_income_statement))
        .route("/cash-flow", get(generate_cash_flow_statement))
        .route("/trial-balance", get(generate_trial_balance))
        .route("/financial-summary", get(generate_financial_summary))
        .route("/comparative-analysis", get(generate_comparative_report))
        .with_state(app_state);

    let bind_addr = std::env::var("REPORTING_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3009".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Reporting service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}