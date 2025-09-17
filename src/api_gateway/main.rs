// src/api_gateway/main.rs - Updated for Axum 0.8 with corrected service mappings
use axum::{
    body::Body,
    extract::{Request, State, Path},
    http::{header, HeaderValue, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response, Json},
    routing::{any, get, post},
    Router,
};
use dashmap::DashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap, 
    sync::Arc, 
    time::{Duration, Instant}
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, 
    timeout::TimeoutLayer, 
    trace::TraceLayer,
    services::ServeDir,
};
use tracing::{error, info, warn, debug};
use uuid::Uuid;

#[derive(Clone, Debug)]
struct ServiceHealth {
    url: String,
    last_check: Instant,
    is_healthy: bool,
    response_time: Duration,
    error_count: u32,
    last_error: Option<String>,
    total_checks: u32,
    successful_checks: u32,
}

impl ServiceHealth {
    fn new(url: String) -> Self {
        Self {
            url,
            last_check: Instant::now(),
            is_healthy: false,
            response_time: Duration::from_millis(0),
            error_count: 0,
            last_error: None,
            total_checks: 0,
            successful_checks: 0,
        }
    }

    fn update_success(&mut self, response_time: Duration) {
        self.last_check = Instant::now();
        self.is_healthy = true;
        self.response_time = response_time;
        self.total_checks += 1;
        self.successful_checks += 1;
        self.last_error = None;
        
        // Gradually reduce error count on successful checks
        if self.error_count > 0 {
            self.error_count = self.error_count.saturating_sub(1);
        }
    }

    fn update_failure(&mut self, error: String) {
        self.last_check = Instant::now();
        self.is_healthy = false;
        self.error_count += 1;
        self.total_checks += 1;
        self.last_error = Some(error);
    }

    fn uptime_percentage(&self) -> f64 {
        if self.total_checks == 0 {
            return 100.0;
        }
        (self.successful_checks as f64 / self.total_checks as f64) * 100.0
    }
}

#[derive(Serialize, Deserialize)]
struct ServiceRegistration {
    name: String,
    url: String,
    health_endpoint: String,
    timeout: u64, // in seconds
}

#[derive(Serialize, Deserialize)]
struct HealthCheckResponse {
    service: String,
    status: String,
    timestamp: String,
    response_time_ms: u64,
    version: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ServiceStatus {
    name: String,
    url: String,
    is_healthy: bool,
    last_check: String,
    response_time_ms: u64,
    error_count: u32,
    last_error: Option<String>,
    uptime_percentage: f64,
}

#[derive(Serialize, Deserialize)]
struct ApiError {
    error: bool,
    message: String,
    status_code: u16,
    timestamp: String,
}

impl ApiError {
    fn new(status_code: StatusCode, message: String) -> Self {
        Self {
            error: true,
            message,
            status_code: status_code.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn from_status_code(status_code: StatusCode) -> Self {
        let message = match status_code {
            StatusCode::UNAUTHORIZED => "Authentication required".to_string(),
            StatusCode::FORBIDDEN => "Insufficient permissions".to_string(), 
            StatusCode::NOT_FOUND => "Resource not found".to_string(),
            StatusCode::CONFLICT => "Resource conflict - operation not allowed in current state".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY => "Invalid data provided - please check your input".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR => "Internal server error - please try again later".to_string(),
            StatusCode::BAD_GATEWAY => "Service temporarily unavailable".to_string(),
            StatusCode::SERVICE_UNAVAILABLE => "Service unavailable - maintenance in progress".to_string(),
            StatusCode::GATEWAY_TIMEOUT => "Request timeout - please try again".to_string(),
            _ => format!("An error occurred ({})", status_code.as_u16()),
        };
        Self::new(status_code, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

#[derive(Clone)]
struct AppState {
    http_client: Client,
    service_urls: HashMap<String, String>,
    service_health: Arc<DashMap<String, ServiceHealth>>,
    health_check_interval: Duration,
}

impl AppState {
    async fn check_service_health(&self, service_name: &str) -> bool {
        let start_time = Instant::now();
        
        if let Some(mut service_health) = self.service_health.get_mut(service_name) {
            let health_url = format!("{}/health", service_health.url);
            
            match tokio::time::timeout(
                Duration::from_secs(10),
                self.http_client.get(&health_url).send()
            ).await {
                Ok(Ok(response)) => {
                    let is_healthy = response.status().is_success();
                    let response_time = start_time.elapsed();
                    
                    if is_healthy {
                        service_health.update_success(response_time);
                    } else {
                        service_health.update_failure(format!("HTTP {}", response.status()));
                    }
                    
                    debug!(
                        "Health check for {} completed in {:?}ms - Status: {}",
                        service_name,
                        response_time.as_millis(),
                        if is_healthy { "Healthy" } else { "Unhealthy" }
                    );
                    
                    is_healthy
                },
                Ok(Err(e)) => {
                    service_health.update_failure(format!("Request error: {}", e));
                    false
                },
                Err(_) => {
                    service_health.update_failure("Timeout".to_string());
                    false
                }
            }
        } else {
            false
        }
    }
    
    async fn get_healthy_service_url(&self, service_name: &str) -> Option<String> {
        if let Some(health) = self.service_health.get(service_name) {
            // If health check is recent (within last 30 seconds) and healthy, use it
            if health.last_check.elapsed() < Duration::from_secs(30) && health.is_healthy {
                return Some(health.url.clone());
            }
            
            // If health check is stale, perform a quick check
            if health.last_check.elapsed() > Duration::from_secs(30) {
                drop(health); // Release the lock
                if self.check_service_health(service_name).await {
                    if let Some(health) = self.service_health.get(service_name) {
                        return Some(health.url.clone());
                    }
                }
            }
        }
        
        None
    }
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut overall_health = true;
    let mut service_statuses = Vec::new();
    
    for service_name in state.service_urls.keys() {
        let is_healthy = if let Some(health) = state.service_health.get(service_name) {
            // If last check was recent, use cached result
            if health.last_check.elapsed() < Duration::from_secs(5) {
                health.is_healthy
            } else {
                drop(health); // Release the lock
                state.check_service_health(service_name).await
            }
        } else {
            false
        };
        
        if !is_healthy {
            overall_health = false;
        }
        
        service_statuses.push(json!({
            "name": service_name,
            "healthy": is_healthy
        }));
    }
    
    let status = if overall_health { "healthy" } else { "degraded" };
    let status_code = if overall_health { 
        StatusCode::OK 
    } else { 
        StatusCode::SERVICE_UNAVAILABLE 
    };
    
    (status_code, Json(json!({
        "status": status,
        "service": "api-gateway",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": service_statuses,
        "version": "1.0.0"
    })))
}

async fn get_service_status(
    State(state): State<Arc<AppState>>,
    Path(service_name): Path<String>,
) -> Result<Json<ServiceStatus>, StatusCode> {
    if let Some(health) = state.service_health.get(&service_name) {
        let status = ServiceStatus {
            name: service_name.clone(),
            url: health.url.clone(),
            is_healthy: health.is_healthy,
            last_check: chrono::DateTime::from_timestamp(
                chrono::Utc::now().timestamp() - health.last_check.elapsed().as_secs() as i64, 0
            ).unwrap_or_default().to_rfc3339(),
            response_time_ms: health.response_time.as_millis() as u64,
            error_count: health.error_count,
            last_error: health.last_error.clone(),
            uptime_percentage: health.uptime_percentage(),
        };
        
        Ok(Json(status))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_all_service_statuses(
    State(state): State<Arc<AppState>>
) -> Json<Vec<ServiceStatus>> {
    let mut statuses = Vec::new();
    
    for entry in state.service_health.iter() {
        let service_name = entry.key();
        let health = entry.value();
        
        let status = ServiceStatus {
            name: service_name.clone(),
            url: health.url.clone(),
            is_healthy: health.is_healthy,
            last_check: chrono::DateTime::from_timestamp(
                chrono::Utc::now().timestamp() - health.last_check.elapsed().as_secs() as i64, 0
            ).unwrap_or_default().to_rfc3339(),
            response_time_ms: health.response_time.as_millis() as u64,
            error_count: health.error_count,
            last_error: health.last_error.clone(),
            uptime_percentage: health.uptime_percentage(),
        };
        
        statuses.push(status);
    }
    
    Json(statuses)
}

async fn register_service(
    State(state): State<Arc<AppState>>,
    Json(registration): Json<ServiceRegistration>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Registering service: {} at {}", registration.name, registration.url);
    
    // Add to service health tracking
    let health = ServiceHealth::new(registration.url.clone());
    state.service_health.insert(registration.name.clone(), health);
    
    // Perform initial health check
    let is_healthy = state.check_service_health(&registration.name).await;
    
    Ok(Json(json!({
        "message": format!("Service {} registered successfully", registration.name),
        "initial_health_check": is_healthy,
        "service_id": Uuid::new_v4().to_string()
    })))
}

async fn proxy_request(
    State(state): State<Arc<AppState>>,
    mut req: Request,
) -> Result<Response, Response> {
    let path = req.uri().path();
    let query = req.uri().query().unwrap_or("");
    
    // Determine target service based on path with CORRECTED mappings
    let (service_name, new_path) = match path {
        p if p.starts_with("/api/auth") => ("auth", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/companies") => ("company-management", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/accounts") => ("chart-of-accounts", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/journal-entries") => ("general-ledger", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/trial-balance") => ("general-ledger", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/account-balances") => ("general-ledger", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/audit-logs") => ("general-ledger", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/vendors") => ("accounts-payable", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/invoices") && p.contains("/pay") => ("accounts-payable", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/aging-report") && p.contains("payable") => ("accounts-payable", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/customers") => ("accounts-receivable", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/invoices") && p.contains("/payment") => ("accounts-receivable", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/aging-report") && p.contains("receivable") => ("accounts-receivable", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/invoices") => ("accounts-payable", p.strip_prefix("/api").unwrap_or(p)), // Default to payable for generic invoices
        p if p.starts_with("/api/aging-report") => ("accounts-payable", p.strip_prefix("/api").unwrap_or(p)), // Default aging to payable
        p if p.starts_with("/api/tax") => ("indonesian-tax", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/items") => ("inventory-management", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/transactions") => ("inventory-management", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/stock-report") => ("inventory-management", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/valuation-report") => ("inventory-management", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/stock-adjustment") => ("inventory-management", p.strip_prefix("/api").unwrap_or(p)),
        p if p.starts_with("/api/reports") => ("reporting", p.strip_prefix("/api/reports").unwrap_or(p)),
        _ => {
            let error = ApiError::from_status_code(StatusCode::NOT_FOUND);
            return Err(error.into_response());
        }
    };

    // Get healthy service URL with failover
    let service_url = state.get_healthy_service_url(service_name).await
        .ok_or_else(|| {
            error!("No healthy instance available for service: {}", service_name);
            let error = ApiError::new(StatusCode::SERVICE_UNAVAILABLE, 
                format!("Service {} is currently unavailable", service_name));
            error.into_response()
        })?;

    let target_url = if query.is_empty() {
        format!("{}{}", service_url, new_path)
    } else {
        format!("{}{}?{}", service_url, new_path, query)
    };

    debug!("Proxying request to: {} -> {}", path, target_url);

    let method = match req.method() {
        &Method::GET => reqwest::Method::GET,
        &Method::POST => reqwest::Method::POST,
        &Method::PUT => reqwest::Method::PUT,
        &Method::DELETE => reqwest::Method::DELETE,
        &Method::PATCH => reqwest::Method::PATCH,
        &Method::OPTIONS => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let mut request_builder = state.http_client
        .request(method, &target_url)
        .timeout(Duration::from_secs(30));

    // Forward headers (excluding host and connection headers)
    for (key, value) in req.headers().iter() {
        let key_str = key.as_str();
        if !matches!(key_str.to_lowercase().as_str(), 
            "host" | "connection" | "content-length" | "transfer-encoding") {
            if let Ok(header_value) = value.to_str() {
                request_builder = request_builder.header(key_str, header_value);
            }
        }
    }

    // Forward body for POST/PUT/PATCH requests
    let body = axum::body::to_bytes(req.into_body(), usize::MAX).await
        .map_err(|e| {
            error!("Failed to read request body: {}", e);
            let error = ApiError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR);
            error.into_response()
        })?;

    if !body.is_empty() {
        request_builder = request_builder.body(body);
    }

    let start_time = Instant::now();
    let response = request_builder.send().await
        .map_err(|e| {
            error!("Proxy request failed for service {}: {}", service_name, e);
            // Mark service as unhealthy
            if let Some(mut health) = state.service_health.get_mut(service_name) {
                health.update_failure(e.to_string());
            }
            let error = ApiError::new(StatusCode::BAD_GATEWAY, 
                format!("Failed to communicate with {} service", service_name));
            error.into_response()
        })?;

    let request_duration = start_time.elapsed();
    debug!("Request to {} completed in {:?}", service_name, request_duration);

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    
    let body_bytes = response.bytes().await
        .map_err(|e| {
            error!("Failed to read response body: {}", e);
            let error = ApiError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR);
            error.into_response()
        })?;

    // Update service health based on response
    if status.is_success() {
        if let Some(mut health) = state.service_health.get_mut(service_name) {
            health.update_success(request_duration);
        }
    } else if status.is_server_error() {
        if let Some(mut health) = state.service_health.get_mut(service_name) {
            health.update_failure(format!("HTTP {}", status));
        }
    }

    Ok(Response::builder()
        .status(status)
        .header("X-Proxy-Service", service_name)
        .header("X-Response-Time", format!("{}ms", request_duration.as_millis()))
        .body(Body::from(body_bytes))
        .unwrap())
}

async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, Response> {
    let path = req.uri().path();
    
    // Skip auth for health checks, auth endpoints, and service management
    if path == "/health" 
        || path.starts_with("/api/auth/login") 
        || path.starts_with("/api/auth/register") 
        || path.starts_with("/services/")
        || path.starts_with("/monitoring/")
        || path.ends_with("/health") {
        return Ok(next.run(req).await);
    }

    // Extract and validate token
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or_else(|| {
            warn!("Missing or invalid authorization header for path: {}", path);
            let error = ApiError::from_status_code(StatusCode::UNAUTHORIZED);
            error.into_response()
        })?;

    // Verify token with auth service
    let auth_url = std::env::var("AUTH_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());
    
    let client = reqwest::Client::new();
    let verify_url = format!("{}/verify?token={}", auth_url, token);
    
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        client.get(&verify_url).send()
    ).await
    .map_err(|_| {
        error!("Token verification timeout");
        let error = ApiError::from_status_code(StatusCode::GATEWAY_TIMEOUT);
        error.into_response()
    })?
    .map_err(|e| {
        error!("Failed to verify token with auth service: {}", e);
        let error = ApiError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR);
        error.into_response()
    })?;

    if !response.status().is_success() {
        warn!("Token verification failed with status: {}", response.status());
        let error = ApiError::from_status_code(StatusCode::UNAUTHORIZED);
        return Err(error.into_response());
    }

    // Extract claims from response
    if let Ok(claims) = response.json::<serde_json::Value>().await {
        if let Some(user_id) = claims.get("sub").and_then(|v| v.as_str()) {
            if let Ok(header_value) = HeaderValue::from_str(user_id) {
                req.headers_mut().insert("X-User-ID", header_value);
            }
        }
        if let Some(company_id) = claims.get("company_id").and_then(|v| v.as_str()) {
            if let Ok(header_value) = HeaderValue::from_str(company_id) {
                req.headers_mut().insert("X-Company-ID", header_value);
            }
        }
    }

    Ok(next.run(req).await)
}

async fn start_health_monitor(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(state.health_check_interval);
    
    loop {
        interval.tick().await;
        
        debug!("Starting periodic health checks");
        let mut health_check_tasks = Vec::new();
        
        for service_name in state.service_urls.keys() {
            let state_clone = state.clone();
            let service_name_clone = service_name.clone();
            
            let task = tokio::spawn(async move {
                state_clone.check_service_health(&service_name_clone).await
            });
            
            health_check_tasks.push(task);
        }
        
        // Wait for all health checks to complete
        for task in health_check_tasks {
            let _ = task.await;
        }
        
        debug!("Completed periodic health checks");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter("info,api_gateway=debug")
        .init();

    info!("Starting API Gateway with Service Discovery...");

    // Initialize service URLs with CORRECTED names and environment variable support
    let mut service_urls = HashMap::new();
    service_urls.insert("auth".to_string(), 
        std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3001".to_string()));
    service_urls.insert("company-management".to_string(), 
        std::env::var("COMPANY_MANAGEMENT_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".to_string()));
    service_urls.insert("chart-of-accounts".to_string(), 
        std::env::var("CHART_OF_ACCOUNTS_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3003".to_string()));
    service_urls.insert("general-ledger".to_string(), 
        std::env::var("GENERAL_LEDGER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3004".to_string()));
    service_urls.insert("indonesian-tax".to_string(), 
        std::env::var("INDONESIAN_TAX_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3005".to_string()));
    service_urls.insert("accounts-payable".to_string(), 
        std::env::var("ACCOUNTS_PAYABLE_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3006".to_string()));
    service_urls.insert("accounts-receivable".to_string(), 
        std::env::var("ACCOUNTS_RECEIVABLE_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3007".to_string()));
    service_urls.insert("inventory-management".to_string(), 
        std::env::var("INVENTORY_MANAGEMENT_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3008".to_string()));
    service_urls.insert("reporting".to_string(), 
        std::env::var("REPORTING_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3009".to_string()));

    // Initialize service health tracking
    let service_health = Arc::new(DashMap::new());
    for (name, url) in &service_urls {
        service_health.insert(name.clone(), ServiceHealth::new(url.clone()));
    }

    let health_check_interval = Duration::from_secs(
        std::env::var("HEALTH_CHECK_INTERVAL")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30)
    );

    let app_state = Arc::new(AppState {
        http_client: Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()?,
        service_urls,
        service_health,
        health_check_interval,
    });

    // Start background health monitoring
    let health_monitor_state = app_state.clone();
    tokio::spawn(async move {
        start_health_monitor(health_monitor_state).await;
    });

    // Perform initial health checks
    info!("Performing initial health checks...");
    for service_name in app_state.service_urls.keys() {
        let is_healthy = app_state.check_service_health(service_name).await;
        info!("Initial health check for {}: {}", 
            service_name, 
            if is_healthy { "✅ Healthy" } else { "❌ Unhealthy" }
        );
    }

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/services/register", post(register_service))
        .route("/services/status", get(get_all_service_statuses))
        .route("/services/:name/status", get(get_service_status))
        // Serve monitoring dashboard
        .nest_service(
            "/monitoring",
            ServeDir::new("monitoring").append_index_html_on_directories(true),
        )
        .route("/api/*path", any(proxy_request))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(60)))
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(auth_middleware))
        )
        .with_state(app_state);

    let bind_addr = std::env::var("API_GATEWAY_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("🚀 API Gateway listening on {}", listener.local_addr()?);
    info!("📊 Health monitoring interval: {:?}", health_check_interval);
    info!("🔍 Service discovery endpoints:");
    info!("   GET  /health - Overall system health");
    info!("   GET  /services/status - All service statuses");
    info!("   GET  /services/:name/status - Individual service status");
    info!("   POST /services/register - Register new service");
    info!("   GET  /monitoring/ - Health dashboard");
    info!("🌐 Configured microservices:");
    info!("   auth -> http://localhost:3001");
    info!("   company-management -> http://localhost:3002");
    info!("   chart-of-accounts -> http://localhost:3003");
    info!("   general-ledger -> http://localhost:3004");
    info!("   indonesian-tax -> http://localhost:3005");
    info!("   accounts-payable -> http://localhost:3006");
    info!("   accounts-receivable -> http://localhost:3007");
    info!("   inventory-management -> http://localhost:3008");
    info!("   reporting -> http://localhost:3009");
    
    axum::serve(listener, app).await?;
    Ok(())
}