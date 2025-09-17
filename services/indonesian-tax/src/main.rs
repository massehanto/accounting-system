mod handlers;
mod models;
mod services;
mod utils;

use axum::{routing::{get, post}, Router};
use handlers::*;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    tax_calculator: services::TaxCalculator,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,indonesian_tax=debug")
        .init();

    info!("Starting Indonesian Tax Service...");

    let pool = database::create_database_pool("indonesian-tax").await?;
    let tax_calculator = services::TaxCalculator::new();

    let app_state = Arc::new(AppState { 
        db: pool,
        tax_calculator,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/tax-configurations", post(create_tax_configuration))
        .route("/tax-configurations", get(get_tax_configurations))
        .route("/tax-transactions", post(create_tax_transaction))
        .route("/tax-report", get(get_tax_report))
        .route("/tax-calculations", get(get_tax_calculations))
        .with_state(app_state);

    let bind_addr = std::env::var("INDONESIAN_TAX_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3005".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Indonesian Tax service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}