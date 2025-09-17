mod handlers;
mod models;
mod utils;

use axum::{routing::{get, post}, Router};
use handlers::*;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    jwt_secret: String,
    argon2: argon2::Argon2<'static>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,auth_service=debug")
        .init();

    info!("Starting Auth Service...");

    let pool = database::create_database_pool("auth").await?;
    
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-jwt-secret-change-in-production".to_string());

    let app_state = Arc::new(AppState {
        db: pool,
        jwt_secret,
        argon2: argon2::Argon2::default(),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/verify", get(verify_token))
        .with_state(app_state);

    let bind_addr = std::env::var("AUTH_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3001".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Auth service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}