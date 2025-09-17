use axum::{extract::{Query, State}, http::HeaderMap, response::Json};
use std::{collections::HashMap, sync::Arc};
use crate::{AppState, models::*, services::report_generator::*};
use common::{ServiceResult, extractors::*};

pub async fn generate_balance_sheet(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<FinancialReport>> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_end = params.get("period_end")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let generator = BalanceSheetGenerator::new(&state.service_registry, &state.http_client);
    let report = generator.generate(company_id, user_id, period_end, &headers).await?;
    
    Ok(Json(report))
}

pub async fn generate_income_statement(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<FinancialReport>> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_start = params.get("period_start")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing period_start".to_string()))?;
    
    let period_end = params.get("period_end")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing period_end".to_string()))?;

    let generator = IncomeStatementGenerator::new(&state.service_registry, &state.http_client);
    let report = generator.generate(company_id, user_id, period_start, period_end, &headers).await?;
    
    Ok(Json(report))
}

pub async fn generate_cash_flow_statement(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<FinancialReport>> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_start = params.get("period_start")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing period_start".to_string()))?;
    
    let period_end = params.get("period_end")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing period_end".to_string()))?;

    let generator = CashFlowGenerator::new(&state.service_registry, &state.http_client);
    let report = generator.generate(company_id, user_id, period_start, period_end, &headers).await?;
    
    Ok(Json(report))
}

pub async fn generate_trial_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<FinancialReport>> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_end = params.get("period_end")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let generator = TrialBalanceGenerator::new(&state.service_registry, &state.http_client);
    let report = generator.generate(company_id, user_id, period_end, &headers).await?;
    
    Ok(Json(report))
}

pub async fn generate_financial_summary(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<FinancialReport>> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_start = params.get("period_start")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing period_start".to_string()))?;
    
    let period_end = params.get("period_end")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing period_end".to_string()))?;

    let generator = FinancialSummaryGenerator::new(&state.service_registry, &state.http_client);
    let report = generator.generate(company_id, user_id, period_start, period_end, &headers).await?;
    
    Ok(Json(report))
}

pub async fn generate_comparative_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<FinancialReport>> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let current_start = params.get("current_period_start")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing current_period_start".to_string()))?;
    
    let current_end = params.get("current_period_end")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing current_period_end".to_string()))?;
    
    let prior_start = params.get("prior_period_start")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing prior_period_start".to_string()))?;
    
    let prior_end = params.get("prior_period_end")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| common::ServiceError::Validation("Missing prior_period_end".to_string()))?;

    let generator = ComparativeAnalysisGenerator::new(&state.service_registry, &state.http_client);
    let report = generator.generate(
        company_id, 
        user_id, 
        current_start, 
        current_end, 
        prior_start, 
        prior_end, 
        &headers
    ).await?;
    
    Ok(Json(report))
}