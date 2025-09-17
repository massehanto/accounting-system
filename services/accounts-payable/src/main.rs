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
    vendor_service: services::VendorService,
    invoice_service: services::InvoiceService,
    payment_service: services::PaymentService,
    aging_service: services::AgingService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,accounts_payable=debug")
        .init();

    info!("Starting Accounts Payable Service...");

    let pool = database::create_database_pool("accounts-payable").await?;
    
    let vendor_service = services::VendorService::new(pool.clone());
    let invoice_service = services::InvoiceService::new(pool.clone());
    let payment_service = services::PaymentService::new(pool.clone());
    let aging_service = services::AgingService::new(pool.clone());

    let app_state = Arc::new(AppState {
        db: pool,
        vendor_service,
        invoice_service,
        payment_service,
        aging_service,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/vendors", post(create_vendor))
        .route("/vendors", get(get_vendors))
        .route("/vendors/:id", get(get_vendor))
        .route("/vendors/:id", put(update_vendor))
        .route("/vendors/:id", axum::routing::delete(delete_vendor))
        .route("/vendors/:id/statistics", get(get_vendor_statistics))
        .route("/invoices", post(create_vendor_invoice))
        .route("/invoices", get(get_vendor_invoices))
        .route("/invoices/:id", get(get_vendor_invoice))
        .route("/invoices/:id/status", put(update_invoice_status))
        .route("/invoices/:id/pay", put(pay_vendor_invoice))
        .route("/invoices/:id/payments", get(get_payment_history))
        .route("/payments/:id/reverse", put(reverse_payment))
        .route("/aging-report", get(get_aging_report))
        .with_state(app_state);

    let bind_addr = std::env::var("ACCOUNTS_PAYABLE_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3006".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Accounts Payable service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}