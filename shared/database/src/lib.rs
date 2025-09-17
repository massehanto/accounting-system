//! Database utilities and connection management

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use tracing::info;

pub mod migrations;
pub mod audit;

pub async fn create_database_pool(service_name: &str) -> anyhow::Result<PgPool> {
    let database_url_key = format!("{}_DATABASE_URL", service_name.to_uppercase().replace("-", "_"));
    let database_url = env::var(&database_url_key)
        .map_err(|_| anyhow::anyhow!("{} must be set", database_url_key))?;

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

    info!("Connected to {} database", service_name);
    Ok(pool)
}

pub async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}