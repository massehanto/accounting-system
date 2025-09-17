use axum::{extract::{Path, Query, State}, http::HeaderMap, response::Json};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use crate::{AppState, models::*};
use common::{ServiceResult, extractors::*, PaginationParams};

pub async fn create_vendor_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateVendorInvoiceRequest>,
) -> ServiceResult<Json<VendorInvoice>> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    let invoice = state.invoice_service
        .create_invoice(payload, company_id, user_id)
        .await?;
    
    Ok(Json(invoice))
}

pub async fn get_vendor_invoices(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<Vec<VendorInvoice>>> {
    let company_id = extract_company_id(&headers)?;
    
    let filters = InvoiceFilters {
        status: params.get("status").cloned(),
        vendor_id: params.get("vendor_id").and_then(|id| Uuid::parse_str(id).ok()),
        date_from: params.get("date_from")
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        date_to: params.get("date_to")
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
    };
    
    let pagination = PaginationParams {
        limit: params.get("limit").and_then(|l| l.parse().ok()),
        offset: params.get("offset").and_then(|o| o.parse().ok()),
    };

    let invoices = state.invoice_service
        .get_invoices(company_id, filters, pagination)
        .await?;
    
    Ok(Json(invoices))
}

pub async fn get_vendor_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(invoice_id): Path<Uuid>,
) -> ServiceResult<Json<VendorInvoice>> {
    let company_id = extract_company_id(&headers)?;
    
    let invoice = state.invoice_service
        .get_invoice_by_id(invoice_id, company_id)
        .await?;
    
    Ok(Json(invoice))
}

pub async fn update_invoice_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(invoice_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> ServiceResult<Json<VendorInvoice>> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    let status = params.get("status")
        .ok_or_else(|| common::ServiceError::Validation("Missing status parameter".to_string()))?
        .parse::<InvoiceStatus>()
        .map_err(|e| common::ServiceError::Validation(format!("Invalid status: {}", e)))?;
    
    let invoice = state.invoice_service
        .update_invoice_status(invoice_id, company_id, status, user_id)
        .await?;
    
    Ok(Json(invoice))
}

pub async fn pay_vendor_invoice(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<PaymentRequest>,
) -> ServiceResult<Json<VendorInvoice>> {
    let company_id = extract_company_id(&headers)?;
    let user_id = extract_user_id(&headers)?;
    
    let invoice = state.payment_service
        .process_payment(invoice_id, company_id, payload, user_id)
        .await?;
    
    Ok(Json(invoice))
}