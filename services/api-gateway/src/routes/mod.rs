use axum::{
    routing::{any, get},
    Router, extract::{Path, Request, State}, 
    response::Response, http::StatusCode,
};
use crate::AppState;

pub mod health;
pub use health::health_check;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_auth_service))
}

pub fn company_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_company_service))
}

pub fn account_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_chart_service))
}

pub fn ledger_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_ledger_service))
}

pub fn tax_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_tax_service))
}

pub fn payable_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_payables_service))
}

pub fn receivable_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_receivables_service))
}

pub fn inventory_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_inventory_service))
}

pub fn reporting_routes() -> Router<AppState> {
    Router::new()
        .route("/*path", any(proxy_to_reporting_service))
}

// Proxy handlers
async fn proxy_to_auth_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "auth", &format!("/{}", path), request).await
}

async fn proxy_to_company_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "company", &format!("/{}", path), request).await
}

async fn proxy_to_chart_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "accounts", &format!("/{}", path), request).await
}

async fn proxy_to_ledger_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "ledger", &format!("/{}", path), request).await
}

async fn proxy_to_tax_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "tax", &format!("/{}", path), request).await
}

async fn proxy_to_payables_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "payables", &format!("/{}", path), request).await
}

async fn proxy_to_receivables_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "receivables", &format!("/{}", path), request).await
}

async fn proxy_to_inventory_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "inventory", &format!("/{}", path), request).await
}

async fn proxy_to_reporting_service(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> Result<Response, StatusCode> {
    proxy_request(&state, "reporting", &format!("/{}", path), request).await
}

async fn proxy_request(
    state: &AppState,
    service_name: &str,
    path: &str,
    request: Request,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let headers = request.headers().clone();
    
    // Extract body
    let body = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    match state.service_registry.proxy_request(service_name, path, method, headers, body).await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let body = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            let mut response_builder = Response::builder().status(status);
            
            // Copy headers
            for (key, value) in headers.iter() {
                response_builder = response_builder.header(key, value);
            }

            match response_builder.body(axum::body::Body::from(body)) {
                Ok(response) => Ok(response),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}