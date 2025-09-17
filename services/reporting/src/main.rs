mod handlers;
mod models;
mod services;
mod utils;

use axum::{routing::get, Router};
use handlers::*;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    report_service: services::ReportService,
    financial_report_service: services::FinancialReportService,
    tax_report_service: services::TaxReportService,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,reporting=debug")
        .init();

    info!("Starting Reporting Service...");

    let pool = database::create_database_pool("reporting").await?;
    
    let report_service = services::ReportService::new(pool.clone());
    let financial_report_service = services::FinancialReportService::new(pool.clone());
    let tax_report_service = services::TaxReportService::new(pool.clone());

    let app_state = Arc::new(AppState {
        db: pool,
        report_service,
        financial_report_service,
        tax_report_service,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        // Financial Reports
        .route("/reports/balance-sheet", get(generate_balance_sheet))
        .route("/reports/income-statement", get(generate_income_statement))
        .route("/reports/cash-flow", get(generate_cash_flow))
        .route("/reports/trial-balance", get(generate_trial_balance))
        .route("/reports/general-ledger", get(generate_general_ledger))
        // Indonesian Tax Reports
        .route("/reports/tax/ppn", get(generate_ppn_report))
        .route("/reports/tax/pph21", get(generate_pph21_report))
        .route("/reports/tax/pph23", get(generate_pph23_report))
        // Business Reports
        .route("/reports/aged-receivables", get(generate_aged_receivables))
        .route("/reports/aged-payables", get(generate_aged_payables))
        .route("/reports/inventory-valuation", get(generate_inventory_valuation))
        // Export formats
        .route("/reports/:report_id/pdf", get(export_pdf))
        .route("/reports/:report_id/excel", get(export_excel))
        .route("/reports/:report_id/csv", get(export_csv))
        .with_state(app_state);

    let bind_addr = std::env::var("REPORTING_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3009".to_string());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Reporting service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}