use axum::{extract::State, http::StatusCode, response::Json, extract::Query};
use std::{sync::Arc, collections::HashMap};
use validator::Validate;
use crate::{AppState, models::*, utils::*};
use common::{ServiceError, ServiceResult};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> ServiceResult<Json<serde_json::Value>> {
    if let Err(e) = payload.validate() {
        return Err(ServiceError::Validation(format!("{:?}", e)));
    }

    let password_hash = hash_password(&state.argon2, &payload.password)?;
    let user_id = uuid::Uuid::new_v4();
    let company_uuid = uuid::Uuid::parse_str(&payload.company_id)
        .map_err(|_| ServiceError::Validation("Invalid company ID".to_string()))?;
    
    // Check if user already exists
    let existing_user = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        payload.email
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);

    if existing_user {
        return Err(ServiceError::Conflict("User already exists".to_string()));
    }

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, full_name, company_id, is_active)
        VALUES ($1, $2, $3, $4, $5, true)
        "#,
        user_id,
        payload.email,
        password_hash,
        payload.full_name,
        company_uuid
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "message": "User registered successfully",
        "user_id": user_id
    })))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> ServiceResult<Json<LoginResponse>> {
    if let Err(e) = payload.validate() {
        return Err(ServiceError::Validation(format!("{:?}", e)));
    }

    let user = sqlx::query!(
        "SELECT id, password_hash, company_id, full_name FROM users WHERE email = $1 AND is_active = true",
        payload.email
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ServiceError::Authentication("Invalid credentials".to_string()))?;

    if !verify_password(&state.argon2, &payload.password, &user.password_hash)? {
        return Err(ServiceError::Authentication("Invalid credentials".to_string()));
    }

    let (token, refresh_token) = generate_tokens(&state.jwt_secret, user.id, user.company_id)?;
    
    Ok(Json(LoginResponse {
        token,
        refresh_token,
        user_id: user.id.to_string(),
        company_id: user.company_id.to_string(),
        expires_in: 3600,
    }))
}

pub async fn verify_token(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<Claims>> {
    let token = params.get("token")
        .ok_or_else(|| ServiceError::Validation("Missing token parameter".to_string()))?;

    let claims = verify_jwt_token(&state.jwt_secret, token)?;
    Ok(Json(claims))
}

// Add other auth handlers...