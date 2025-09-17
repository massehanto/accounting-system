mod routes;
mod middleware;
mod services;
mod config;

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tracing::info;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    config: Arc<config::GatewayConfig>,
    service_registry: Arc<services::ServiceRegistry>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,api_gateway=debug")
        .init();

    info!("Starting API Gateway...");

    let config = Arc::new(config::GatewayConfig::from_env()?);
    let service_registry = Arc::new(services::ServiceRegistry::new(&config));

    let app_state = AppState {
        config: config.clone(),
        service_registry,
    };

    let app = Router::new()
        .route("/health", get(routes::health_check))
        
        // Auth routes
        .nest("/api/v1/auth", routes::auth_routes())
        
        // Company management routes
        .nest("/api/v1/companies", routes::company_routes())
        
        // Chart of accounts routes
        .nest("/api/v1/accounts", routes::account_routes())
        
        // General ledger routes
        .nest("/api/v1/ledger", routes::ledger_routes())
        
        // Tax routes
        .nest("/api/v1/tax", routes::tax_routes())
        
        // Accounts payable routes
        .nest("/api/v1/payables", routes::payable_routes())
        
        // Accounts receivable routes
        .nest("/api/v1/receivables", routes::receivable_routes())
        
        // Inventory routes
        .nest("/api/v1/inventory", routes::inventory_routes())
        
        // Reporting routes
        .nest("/api/v1/reports", routes::reporting_routes())
        
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::auth_middleware
                ))
        )
        .with_state(app_state);

    let bind_addr = std::env::var("API_GATEWAY_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("API Gateway listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}