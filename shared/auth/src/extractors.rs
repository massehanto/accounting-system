use crate::models::AuthContext;
use axum::{
    async_trait,
    extract::{FromRequestParts, rejection::ExtensionRejection},
    http::{request::Parts, StatusCode},
};

// Extractor for required authentication
pub struct RequireAuth(pub AuthContext);

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_context = parts
            .extensions
            .get::<AuthContext>()
            .ok_or(StatusCode::UNAUTHORIZED)?
            .clone();

        Ok(RequireAuth(auth_context))
    }
}

// Extractor for optional authentication
pub struct OptionalAuth(pub Option<AuthContext>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
{
    type Rejection = (); // Never fails

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_context = parts.extensions.get::<AuthContext>().cloned();
        Ok(OptionalAuth(auth_context))
    }
}

// Convenience functions for use in handlers
use axum::http::HeaderMap;
use uuid::Uuid;

pub fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    // This would typically be used with the auth middleware
    // For now, extract from a custom header (in real implementation, use the auth context)
    headers
        .get("x-user-id")
        .and_then(|header| header.to_str().ok())
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

pub fn extract_company_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    headers
        .get("x-company-id")
        .and_then(|header| header.to_str().ok())
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

// Better approach using auth context from middleware
pub fn extract_user_id_from_context(auth: &AuthContext) -> Uuid {
    auth.user.id
}

pub fn extract_company_id_from_context(auth: &AuthContext) -> Uuid {
    auth.user.company_id
}