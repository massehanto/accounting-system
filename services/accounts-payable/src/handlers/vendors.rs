use axum::{extract::{Path, Query, State}, http::HeaderMap, response::Json};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use crate::{AppState, models::*};
use common::{ServiceResult, extractors::*, PaginationParams};

pub async fn create_vendor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateVendorRequest>,
) -> ServiceResult<Json<Vendor>> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    let vendor = state.vendor_service
        .create_vendor(payload, company_id, user_id)
        .await?;
    
    Ok(Json(vendor))
}

pub async fn get_vendors(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<Vec<Vendor>>> {
    let company_id = extract_company_id(&headers)?;
    
    let include_inactive = params.get("include_inactive")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    let search_term = params.get("search");
    let pagination = PaginationParams {
        limit: params.get("limit").and_then(|l| l.parse().ok()),
        offset: params.get("offset").and_then(|o| o.parse().ok()),
    };

    let vendors = state.vendor_service
        .get_vendors(company_id, include_inactive, search_term, pagination)
        .await?;
    
    Ok(Json(vendors))
}

pub async fn get_vendor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(vendor_id): Path<Uuid>,
) -> ServiceResult<Json<Vendor>> {
    let company_id = extract_company_id(&headers)?;
    
    let vendor = state.vendor_service
        .get_vendor_by_id(vendor_id, company_id)
        .await?;
    
    Ok(Json(vendor))
}

pub async fn update_vendor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(vendor_id): Path<Uuid>,
    Json(payload): Json<UpdateVendorRequest>,
) -> ServiceResult<Json<Vendor>> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    let vendor = state.vendor_service
        .update_vendor(vendor_id, payload, company_id, user_id)
        .await?;
    
    Ok(Json(vendor))
}

pub async fn delete_vendor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(vendor_id): Path<Uuid>,
) -> ServiceResult<()> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    state.vendor_service
        .delete_vendor(vendor_id, company_id, user_id)
        .await?;
    
    Ok(())
}