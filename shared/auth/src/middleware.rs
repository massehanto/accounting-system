use crate::{jwt::JwtManager, models::{AuthContext, AuthUser, Claims}};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract token from "Bearer <token>"
    let token = JwtManager::extract_token_from_header(auth_header)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify token
    let jwt_manager = JwtManager::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let claims = jwt_manager
        .verify_access_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Create AuthUser from claims
    let user = AuthUser {
        id: Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        email: claims.email.clone(),
        full_name: claims.full_name.clone(),
        company_id: Uuid::parse_str(&claims.company_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        is_active: true, // Assume active if token is valid
    };

    // Create auth context
    let auth_context = AuthContext {
        user,
        permissions: vec![], // Would be populated from database in real implementation
        token_jti: claims.jti,
    };

    // Add auth context to request extensions
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

pub async fn optional_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and verify token, but don't fail if it's missing
    if let Some(auth_header) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if let Some(token) = JwtManager::extract_token_from_header(auth_header) {
            if let Ok(jwt_manager) = JwtManager::new() {
                if let Ok(claims) = jwt_manager.verify_access_token(token) {
                    if let (Ok(user_id), Ok(company_id)) = (
                        Uuid::parse_str(&claims.sub),
                        Uuid::parse_str(&claims.company_id),
                    ) {
                        let user = AuthUser {
                            id: user_id,
                            email: claims.email.clone(),
                            full_name: claims.full_name.clone(),
                            company_id,
                            is_active: true,
                        };

                        let auth_context = AuthContext {
                            user,
                            permissions: vec![],
                            token_jti: claims.jti,
                        };

                        request.extensions_mut().insert(auth_context);
                    }
                }
            }
        }
    }

    next.run(request).await
}

// Role-based middleware
pub fn require_role(required_role: &'static str) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    move |request: Request, next: Next| async move {
        let auth_context = request
            .extensions()
            .get::<AuthContext>()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Check if user has required role/permission
        if !auth_context.permissions.contains(&required_role.to_string()) {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(next.run(request).await)
    }
}