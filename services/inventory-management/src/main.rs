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
    inventory_service: services::InventoryService,
    audit_logger: database::audit::AuditLogger,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,inventory_service=debug")
        .init();

    info!("Starting Inventory Management Service...");

    let pool = database::create_database_pool("inventory-management").await?;
    let inventory_service = services::InventoryService::new(pool.clone());
    let audit_logger = database::audit::AuditLogger::new(pool.clone());

    let app_state = Arc::new(AppState { 
        db: pool,
        inventory_service,
        audit_logger,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/items", post(create_inventory_item))
        .route("/items", get(get_inventory_items))
        .route("/items/:id", get(get_inventory_item))
        .route("/items/:id", put(update_inventory_item))
        .route("/transactions", post(create_inventory_transaction))
        .route("/transactions", get(get_inventory_transactions))
        .route("/stock-adjustment", post(adjust_stock))
        .route("/stock-report", get(get_stock_report))
        .route("/valuation-report", get(get_valuation_report))
        .with_state(app_state);

    let bind_addr = std::env::var("INVENTORY_MANAGEMENT_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3008".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Inventory Management service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}