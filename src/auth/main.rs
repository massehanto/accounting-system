// src/auth/main.rs
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{sync::Arc, env};
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company_id: String,
    exp: i64,
    iat: i64,
    jti: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    refresh_token: String,
    user_id: String,
    company_id: String,
    expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
    #[validate(length(min = 2, message = "Full name must be at least 2 characters"))]
    full_name: String,
    company_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
    jwt_secret: String,
    argon2: Argon2<'static>,
}

// Database connection function specific to auth service
async fn create_auth_database_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("AUTH_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("AUTH_DATABASE_URL must be set"))?;

    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "20".to_string())
        .parse::<u32>()
        .unwrap_or(20);

    let min_connections = env::var("DB_MIN_CONNECTIONS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u32>()
        .unwrap_or(5);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .connect(&database_url)
        .await?;

    info!("Connected to auth database: {}", database_url.split('@').last().unwrap_or("unknown"));
    Ok(pool)
}

async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

// Rest of the auth service implementation remains the same...
async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Err(e) = payload.validate() {
        warn!("Registration validation failed: {:?}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = state.argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| {
            error!("Failed to hash password: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_string();

    let user_id = Uuid::new_v4();
    let company_uuid = Uuid::parse_str(&payload.company_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Database error checking existing user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let result = sqlx::query!(
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
    .await;

    match result {
        Ok(_) => {
            info!("User registered successfully: {}", payload.email);
            Ok(Json(serde_json::json!({
                "message": "User registered successfully",
                "user_id": user_id
            })))
        },
        Err(e) => {
            error!("Failed to register user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    if let Err(e) = payload.validate() {
        warn!("Login validation failed: {:?}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = sqlx::query!(
        "SELECT id, password_hash, company_id, full_name FROM users WHERE email = $1 AND is_active = true",
        payload.email
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Database error during login: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = user.ok_or_else(|| {
        warn!("Login attempt with non-existent email: {}", payload.email);
        StatusCode::UNAUTHORIZED
    })?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| {
            error!("Failed to parse password hash: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let is_valid = state.argon2
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        warn!("Invalid password attempt for email: {}", payload.email);
        return Err(StatusCode::UNAUTHORIZED);
    }

    let now = Utc::now();
    let expires_in = 3600;
    let refresh_expires_in = 86400 * 7;
    let exp = now + Duration::seconds(expires_in);
    let refresh_exp = now + Duration::seconds(refresh_expires_in);
    
    let jti = Uuid::new_v4().to_string();
    let refresh_jti = Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user.id.to_string(),
        company_id: user.company_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        jti: jti.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| {
        error!("Failed to create JWT token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let refresh_claims = Claims {
        sub: user.id.to_string(),
        company_id: user.company_id.to_string(),
        exp: refresh_exp.timestamp(),
        iat: now.timestamp(),
        jti: refresh_jti.clone(),
    };

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| {
        error!("Failed to create refresh token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "INSERT INTO refresh_tokens (jti, user_id, expires_at, created_at) VALUES ($1, $2, $3, $4)",
        refresh_jti,
        user.id,
        refresh_exp.naive_utc(),
        now.naive_utc()
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to store refresh token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Successful login for user: {}", payload.email);

    Ok(Json(LoginResponse {
        token,
        refresh_token,
        user_id: user.id.to_string(),
        company_id: user.company_id.to_string(),
        expires_in,
    }))
}

async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let token_data = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let refresh_token_record = sqlx::query!(
        "SELECT user_id FROM refresh_tokens WHERE jti = $1 AND expires_at > NOW()",
        token_data.claims.jti
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Database error checking refresh token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let _refresh_record = refresh_token_record.ok_or(StatusCode::UNAUTHORIZED)?;

    let user = sqlx::query!(
        "SELECT id, company_id, email FROM users WHERE id = $1 AND is_active = true",
        Uuid::parse_str(&token_data.claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Database error fetching user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let now = Utc::now();
    let expires_in = 3600;
    let exp = now + Duration::seconds(expires_in);
    let new_jti = Uuid::new_v4().to_string();

    let new_claims = Claims {
        sub: user.id.to_string(),
        company_id: user.company_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        jti: new_jti,
    };

    let new_token = encode(
        &Header::default(),
        &new_claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| {
        error!("Failed to create new JWT token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(LoginResponse {
        token: new_token,
        refresh_token: payload.refresh_token,
        user_id: user.id.to_string(),
        company_id: user.company_id.to_string(),
        expires_in,
    }))
}

async fn verify_token(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Claims>, StatusCode> {
    let token = params.get("token").ok_or(StatusCode::BAD_REQUEST)?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        warn!("Token verification failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    Ok(Json(token_data.claims))
}

async fn logout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token_data = decode::<Claims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    sqlx::query!(
        "DELETE FROM refresh_tokens WHERE jti = $1",
        token_data.claims.jti
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to delete refresh token: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_healthy = check_database_health(&state.db).await;
    
    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy { 
        StatusCode::OK 
    } else { 
        StatusCode::SERVICE_UNAVAILABLE 
    };
    
    (status_code, Json(serde_json::json!({
        "service": "auth-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,auth_service=debug")
        .init();

    info!("Starting Auth Service...");

    let pool = create_auth_database_pool().await?;

    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            warn!("JWT_SECRET not set, using default (NOT FOR PRODUCTION)");
            "default-jwt-secret-change-in-production-minimum-32-characters".to_string()
        });

    if jwt_secret.len() < 32 {
        warn!("JWT_SECRET should be at least 32 characters for security");
    }

    let argon2 = Argon2::default();

    let app_state = Arc::new(AppState {
        db: pool,
        jwt_secret,
        argon2,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/verify", get(verify_token))
        .with_state(app_state);

    let bind_addr = env::var("AUTH_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3001".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Auth service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}