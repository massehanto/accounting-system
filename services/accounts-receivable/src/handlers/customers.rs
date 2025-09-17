use axum::{extract::{Path, Query, State}, http::HeaderMap, response::Json};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use crate::{AppState, models::*};
use common::{ServiceResult, extractors::*, PaginationParams};

pub async fn create_customer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateCustomerRequest>,
) -> ServiceResult<Json<Customer>> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    let customer = state.customer_service
        .create_customer(payload, company_id, user_id)
        .await?;
    
    Ok(Json(customer))
}

pub async fn get_customers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<Vec<Customer>>> {
    let company_id = extract_company_id(&headers)?;
    
    let include_inactive = params.get("include_inactive")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    let search_term = params.get("search");
    let pagination = PaginationParams {
        limit: params.get("limit").and_then(|l| l.parse().ok()),
        offset: params.get("offset").and_then(|o| o.parse().ok()),
    };

    let customers = state.customer_service
        .get_customers(company_id, include_inactive, search_term, pagination)
        .await?;
    
    Ok(Json(customers))
}

pub async fn get_customer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(customer_id): Path<Uuid>,
) -> ServiceResult<Json<Customer>> {
    let company_id = extract_company_id(&headers)?;
    
    let customer = state.customer_service
        .get_customer_by_id(customer_id, company_id)
        .await?;
    
    Ok(Json(customer))
}

pub async fn update_customer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(customer_id): Path<Uuid>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> ServiceResult<Json<Customer>> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    let customer = state.customer_service
        .update_customer(customer_id, payload, company_id, user_id)
        .await?;
    
    Ok(Json(customer))
}

pub async fn get_customer_credit_info(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(customer_id): Path<Uuid>,
) -> ServiceResult<Json<CustomerCreditInfo>> {
    let company_id = extract_company_id(&headers)?;
    
    let credit_info = state.customer_service
        .get_customer_credit_info(customer_id, company_id)
        .await?;
    
    Ok(Json(credit_info))
}