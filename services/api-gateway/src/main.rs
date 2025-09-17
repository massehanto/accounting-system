mod service_discovery;
mod proxy;
mod middleware;
mod health;

use axum::{routing::{any, get, post}, Router, middleware as axum_middleware};
use service_discovery::*;
use proxy::*;
use middleware::*;
use health::*;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter("info,api_gateway=debug")
        .init();

    let service_registry = Arc::new(ServiceRegistry::new().await?);
    
    // Start health monitoring
    let health_monitor = service_registry.clone();
    tokio::spawn(async move {
        health_monitor.start_health_monitoring().await;
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/services/register", post(register_service))
        .route("/services/status", get(get_all_service_statuses))
        .route("/api/*path", any(proxy_request))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(std::time::Duration::from_secs(60)))
                .layer(CorsLayer::permissive())
                .layer(axum_middleware::from_fn_with_state(
                    service_registry.clone(),
                    auth_middleware
                ))
        )
        .with_state(service_registry);

    let bind_addr = std::env::var("API_GATEWAY_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("API Gateway listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}