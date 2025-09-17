use axum::{http::StatusCode, response::{IntoResponse, Json}};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ServiceError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServiceError::Authentication(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServiceError::Authorization(_) => (StatusCode::FORBIDDEN, self.to_string()),
            ServiceError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServiceError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            ServiceError::ExternalService(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            ServiceError::Database(_) | ServiceError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": true,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        (status, body).into_response()
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;