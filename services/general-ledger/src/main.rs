mod handlers;
mod models;
mod services;
mod utils;

use axum::{routing::{get, post, put}, Router};
use handlers::*;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    audit_logger: database::audit::AuditLogger,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,general_ledger=debug")
        .init();

    info!("Starting General Ledger Service...");

    let pool = database::create_database_pool("general-ledger").await?;
    let audit_logger = database::audit::AuditLogger::new(pool.clone());

    let app_state = Arc::new(AppState { 
        db: pool,
        audit_logger,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/journal-entries", post(create_journal_entry))
        .route("/journal-entries", get(get_journal_entries))
        .route("/journal-entries/:id", get(get_journal_entry_with_lines))
        .route("/journal-entries/:id", axum::routing::delete(delete_journal_entry))
        .route("/journal-entries/:id/status", put(update_journal_entry_status))
        .route("/trial-balance", get(get_trial_balance))
        .route("/account-balances", get(get_account_balances))
        .with_state(app_state);

    let bind_addr = std::env::var("GENERAL_LEDGER_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3004".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("General Ledger service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}