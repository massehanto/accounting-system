use axum::http::{HeaderMap, StatusCode};
use uuid::Uuid;
use crate::ServiceError;

pub fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, ServiceError> {
    headers
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| ServiceError::Authentication("Missing or invalid user ID".to_string()))
}

pub fn extract_company_id(headers: &HeaderMap) -> Result<Uuid, ServiceError> {
    headers
        .get("X-Company-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| ServiceError::Authentication("Missing or invalid company ID".to_string()))
}