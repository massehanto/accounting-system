use axum::{extract::{Query, State}, http::HeaderMap, response::Json};
use std::{collections::HashMap, sync::Arc};
use crate::{AppState, models::*};
use common::{ServiceResult, extractors::*};

pub async fn generate_balance_sheet(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<BalanceSheetReport>> {
    let company_id = extract_company_id(&headers)?;
    
    let as_of_date = params.get("as_of_date")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let report = state.financial_report_service
        .generate_balance_sheet(company_id, as_of_date)
        .await?;
    
    Ok(Json(report))
}

pub async fn generate_income_statement(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<IncomeStatementReport>> {
    let company_id = extract_company_id(&headers)?;
    
    let end_date = chrono::Utc::now().date_naive();
    let start_date = params.get("start_date")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(end_date.year(), 1, 1).unwrap());
    
    let end_date = params.get("end_date")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or(end_date);

    let report = state.financial_report_service
        .generate_income_statement(company_id, start_date, end_date)
        .await?;
    
    Ok(Json(report))
}

pub async fn generate_trial_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<TrialBalanceReport>> {
    let company_id = extract_company_id(&headers)?;
    
    let as_of_date = params.get("as_of_date")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let report = state.financial_report_service
        .generate_trial_balance(company_id, as_of_date)
        .await?;
    
    Ok(Json(report))
}

pub async fn generate_cash_flow(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<serde_json::Value>> {
    let company_id = extract_company_id(&headers)?;
    
    // Cash flow statement implementation would go here
    // This is a placeholder
    Ok(Json(serde_json::json!({
        "company_id": company_id,
        "message": "Cash flow statement not yet implemented"
    })))
}

pub async fn generate_general_ledger(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<serde_json::Value>> {
    let company_id = extract_company_id(&headers)?;
    
    // General ledger report implementation would go here
    // This is a placeholder
    Ok(Json(serde_json::json!({
        "company_id": company_id,
        "message": "General ledger report not yet implemented"
    })))
}