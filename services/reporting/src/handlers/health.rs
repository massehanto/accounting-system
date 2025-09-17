use axum::response::Json;
use common::health::health_check_handler;

pub async fn health_check() -> Result<Json<common::ServiceInfo>, axum::http::StatusCode> {
    health_check_handler("reporting-service", "1.0.0", vec![]).await
}