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
    customer_service: services::CustomerService,
    invoice_service: services::InvoiceService,
    payment_service: services::PaymentService,
    aging_service: services::AgingService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,accounts_receivable=debug")
        .init();

    info!("Starting Accounts Receivable Service...");

    let pool = database::create_database_pool("accounts-receivable").await?;
    
    let customer_service = services::CustomerService::new(pool.clone());
    let invoice_service = services::InvoiceService::new(pool.clone());
    let payment_service = services::PaymentService::new(pool.clone());
    let aging_service = services::AgingService::new(pool.clone());

    let app_state = Arc::new(AppState {
        db: pool,
        customer_service,
        invoice_service,
        payment_service,
        aging_service,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/customers", post(create_customer))
        .route("/customers", get(get_customers))
        .route("/customers/:id", get(get_customer))
        .route("/customers/:id", put(update_customer))
        .route("/customers/:id/credit-info", get(get_customer_credit_info))
        .route("/customers/:id/statistics", get(get_customer_statistics))
        .route("/invoices", post(create_customer_invoice))
        .route("/invoices", get(get_customer_invoices))
        .route("/invoices/:id", get(get_customer_invoice))
        .route("/invoices/:id/status", put(update_invoice_status))
        .route("/invoices/:id/payment", put(receive_payment))
        .route("/invoices/:id/payments", get(get_payment_history))
        .route("/aging-report", get(get_customer_aging_report))
        .route("/credit-limit-check", post(check_credit_limit))
        .with_state(app_state);

    let bind_addr = std::env::var("ACCOUNTS_RECEIVABLE_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3007".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Accounts Receivable service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}