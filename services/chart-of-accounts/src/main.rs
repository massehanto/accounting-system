mod handlers;
mod models;
mod services;
mod utils;

use axum::{routing::{delete, get, post, put}, Router};
use handlers::*;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    chart_service: services::ChartService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,chart_of_accounts=debug")
        .init();

    info!("Starting Chart of Accounts Service...");

    let pool = database::create_database_pool("chart-of-accounts").await?;
    let chart_service = services::ChartService::new(pool.clone());

    let app_state = Arc::new(AppState { 
        db: pool,
        chart_service,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/accounts", post(create_account))
        .route("/accounts", get(get_accounts))
        .route("/accounts/:id", get(get_account))
        .route("/accounts/:id", put(update_account))
        .route("/accounts/:id", delete(delete_account))
        .route("/accounts/template/:template_name", post(create_from_template))
        .route("/accounts/validate", post(validate_account_structure))
        .with_state(app_state);

    let bind_addr = std::env::var("CHART_OF_ACCOUNTS_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3003".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Chart of Accounts service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}