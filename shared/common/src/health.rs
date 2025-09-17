use axum::{response::Json, http::StatusCode};
use serde_json::json;
use crate::ServiceInfo;

pub async fn health_check_handler(
    service_name: &str,
    version: &str,
    additional_checks: Vec<(&str, bool)>,
) -> Result<Json<ServiceInfo>, StatusCode> {
    let all_healthy = additional_checks.iter().all(|(_, healthy)| *healthy);
    
    let status = if all_healthy { "healthy" } else { "unhealthy" };
    let status_code = if all_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    
    let mut response_data = json!({
        "service": service_name,
        "version": version,
        "status": status,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // Add additional check details
    if !additional_checks.is_empty() {
        let checks: Vec<_> = additional_checks
            .into_iter()
            .map(|(name, healthy)| json!({
                "name": name,
                "status": if healthy { "ok" } else { "failed" }
            }))
            .collect();
        response_data["checks"] = json!(checks);
    }

    if all_healthy {
        Ok(Json(serde_json::from_value(response_data).unwrap()))
    } else {
        Err(status_code)
    }
}