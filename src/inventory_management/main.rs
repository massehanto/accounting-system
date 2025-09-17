// src/inventory/main.rs
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::{Json, IntoResponse},
    routing::{get, post, put},
    Router,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type, Transaction, Postgres, postgres::PgPoolOptions};
use std::{sync::Arc, env};
use tracing::{error, info, warn};
use uuid::Uuid;

// Helper functions for extracting user info from headers
fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    headers
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

fn extract_company_id(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    headers
        .get("X-Company-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

// Database connection function specific to inventory service
async fn create_inventory_database_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("INVENTORY_MANAGEMENT_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("INVENTORY_MANAGEMENT_DATABASE_URL must be set"))?;

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

    info!("Connected to inventory database: {}", database_url.split('@').last().unwrap_or("unknown"));
    Ok(pool)
}

async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "item_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemType {
    Raw,
    Finished,
    Service,
}

impl std::str::FromStr for ItemType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "RAW" => Ok(ItemType::Raw),
            "FINISHED" => Ok(ItemType::Finished),
            "SERVICE" => Ok(ItemType::Service),
            _ => Err(format!("Invalid item type: {}", s))
        }
    }
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Raw => write!(f, "RAW"),
            ItemType::Finished => write!(f, "FINISHED"),
            ItemType::Service => write!(f, "SERVICE"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InventoryItem {
    id: Uuid,
    company_id: Uuid,
    item_code: String,
    item_name: String,
    description: Option<String>,
    item_type: ItemType,
    unit_of_measure: String,
    unit_cost: Decimal,
    selling_price: Decimal,
    quantity_on_hand: Decimal,
    reorder_level: Decimal,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InventoryTransaction {
    id: Uuid,
    company_id: Uuid,
    item_id: Uuid,
    item_code: Option<String>,
    item_name: Option<String>,
    transaction_type: String,
    transaction_date: NaiveDate,
    quantity: Decimal,
    unit_cost: Decimal,
    total_cost: Decimal,
    reference: Option<String>,
    journal_entry_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateInventoryItemRequest {
    company_id: Uuid,
    item_code: String,
    item_name: String,
    description: Option<String>,
    item_type: ItemType,
    unit_of_measure: String,
    unit_cost: Decimal,
    selling_price: Decimal,
    reorder_level: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateInventoryItemRequest {
    item_name: String,
    description: Option<String>,
    unit_cost: Decimal,
    selling_price: Decimal,
    reorder_level: Decimal,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateInventoryTransactionRequest {
    company_id: Uuid,
    item_id: Uuid,
    transaction_type: String, // "IN" or "OUT"
    transaction_date: NaiveDate,
    quantity: Decimal,
    unit_cost: Decimal,
    reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StockAdjustmentRequest {
    item_id: Uuid,
    adjustment_quantity: Decimal, // Can be negative
    reason: String,
    reference: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LowStockAlert {
    item_id: Uuid,
    item_code: String,
    item_name: String,
    current_quantity: Decimal,
    reorder_level: Decimal,
    shortage_amount: Decimal,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

impl AppState {
    async fn log_inventory_audit(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        item_id: Uuid,
        action: &str,
        old_quantity: Option<Decimal>,
        new_quantity: Option<Decimal>,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let audit_data = serde_json::json!({
            "action": action,
            "old_quantity": old_quantity,
            "new_quantity": new_quantity,
            "timestamp": chrono::Utc::now()
        });

        sqlx::query!(
            r#"
            INSERT INTO audit_logs (id, table_name, record_id, action, old_values, new_values, user_id, timestamp)
            VALUES ($1, 'inventory_items', $2, $3, $4, $4, $5, NOW())
            "#,
            Uuid::new_v4(),
            item_id,
            action,
            Some(audit_data.clone()),
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_healthy = check_database_health(&state.db).await;
    
    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy { 
        StatusCode::OK 
    } else { 
        StatusCode::SERVICE_UNAVAILABLE 
    };
    
    (status_code, Json(serde_json::json!({
        "service": "inventory-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    })))
}

async fn create_inventory_item(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateInventoryItemRequest>,
) -> Result<Json<InventoryItem>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    payload.company_id = company_id;
    
    // Validate item code uniqueness
    let existing_item = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM inventory_items WHERE company_id = $1 AND item_code = $2)",
        company_id,
        payload.item_code
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to check item code uniqueness: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .unwrap_or(false);

    if existing_item {
        warn!("Attempt to create item with duplicate code: {}", payload.item_code);
        return Err(StatusCode::CONFLICT);
    }
    
    let item_id = Uuid::new_v4();
    
    let item = sqlx::query_as!(
        InventoryItem,
        r#"
        INSERT INTO inventory_items 
        (id, company_id, item_code, item_name, description, item_type, unit_of_measure, unit_cost, selling_price, reorder_level)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, company_id, item_code, item_name, description, 
                  item_type as "item_type: ItemType", unit_of_measure, 
                  unit_cost, selling_price, quantity_on_hand, reorder_level, is_active,
                  created_at, updated_at
        "#,
        item_id,
        payload.company_id,
        payload.item_code,
        payload.item_name,
        payload.description,
        payload.item_type as ItemType,
        payload.unit_of_measure,
        payload.unit_cost,
        payload.selling_price,
        payload.reorder_level.unwrap_or(Decimal::from(0))
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create inventory item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created inventory item {} for company {} by user {}", 
        payload.item_code, company_id, user_id);

    Ok(Json(item))
}

async fn get_inventory_items(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<InventoryItem>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    
    let include_inactive = params.get("include_inactive")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    let limit: i64 = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(100)
        .min(1000);

    let offset: i64 = params.get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    let items = if include_inactive {
        sqlx::query_as!(
            InventoryItem,
            r#"
            SELECT id, company_id, item_code, item_name, description, 
                   item_type as "item_type: ItemType", unit_of_measure, 
                   unit_cost, selling_price, quantity_on_hand, reorder_level, is_active,
                   created_at, updated_at
            FROM inventory_items 
            WHERE company_id = $1
            ORDER BY item_code
            LIMIT $2 OFFSET $3
            "#,
            company_id,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as!(
            InventoryItem,
            r#"
            SELECT id, company_id, item_code, item_name, description, 
                   item_type as "item_type: ItemType", unit_of_measure, 
                   unit_cost, selling_price, quantity_on_hand, reorder_level, is_active,
                   created_at, updated_at
            FROM inventory_items 
            WHERE company_id = $1 AND is_active = true
            ORDER BY item_code
            LIMIT $2 OFFSET $3
            "#,
            company_id,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await
    };

    let items = items.map_err(|e| {
        error!("Failed to fetch inventory items: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(items))
}

async fn get_inventory_item(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<InventoryItem>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let item = sqlx::query_as!(
        InventoryItem,
        r#"
        SELECT id, company_id, item_code, item_name, description, 
               item_type as "item_type: ItemType", unit_of_measure, 
               unit_cost, selling_price, quantity_on_hand, reorder_level, is_active,
               created_at, updated_at
        FROM inventory_items 
        WHERE id = $1 AND company_id = $2
        "#,
        id,
        company_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch inventory item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(item))
}

async fn update_inventory_item(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateInventoryItemRequest>,
) -> Result<Json<InventoryItem>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;

    let item = sqlx::query_as!(
        InventoryItem,
        r#"
        UPDATE inventory_items 
        SET item_name = $1, description = $2, unit_cost = $3, selling_price = $4, 
            reorder_level = $5, is_active = $6, updated_at = NOW()
        WHERE id = $7 AND company_id = $8
        RETURNING id, company_id, item_code, item_name, description, 
                  item_type as "item_type: ItemType", unit_of_measure, 
                  unit_cost, selling_price, quantity_on_hand, reorder_level, is_active,
                  created_at, updated_at
        "#,
        payload.item_name,
        payload.description,
        payload.unit_cost,
        payload.selling_price,
        payload.reorder_level,
        payload.is_active,
        id,
        company_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to update inventory item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    info!("Updated inventory item {} by user {}", item.item_code, user_id);

    Ok(Json(item))
}

async fn create_inventory_transaction(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateInventoryTransactionRequest>,
) -> Result<Json<InventoryTransaction>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    payload.company_id = company_id;
    
    // Validate transaction type
    if !matches!(payload.transaction_type.as_str(), "IN" | "OUT") {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate quantity is positive
    if payload.quantity <= Decimal::ZERO {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get current item details and verify ownership
    let item = sqlx::query!(
        r#"
        SELECT item_code, item_name, quantity_on_hand 
        FROM inventory_items 
        WHERE id = $1 AND company_id = $2 AND is_active = true
        "#,
        payload.item_id,
        company_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to fetch item details: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check for sufficient inventory on OUT transactions
    if payload.transaction_type == "OUT" && item.quantity_on_hand < payload.quantity {
        warn!("Insufficient inventory for item {}: requested {}, available {}", 
            item.item_code, payload.quantity, item.quantity_on_hand);
        return Err(StatusCode::CONFLICT);
    }

    let transaction_id = Uuid::new_v4();
    let total_cost = payload.quantity * payload.unit_cost;

    // Create transaction record
    let transaction = sqlx::query_as!(
        InventoryTransaction,
        r#"
        INSERT INTO inventory_transactions 
        (id, company_id, item_id, transaction_type, transaction_date, quantity, unit_cost, total_cost, reference)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, company_id, item_id, transaction_type, transaction_date, 
                  quantity, unit_cost, total_cost, reference, journal_entry_id, created_at
        "#,
        transaction_id,
        payload.company_id,
        payload.item_id,
        payload.transaction_type,
        payload.transaction_date,
        payload.quantity,
        payload.unit_cost,
        total_cost,
        payload.reference
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to create inventory transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update inventory quantity
    let quantity_change = if payload.transaction_type == "IN" {
        payload.quantity
    } else {
        -payload.quantity
    };

    let new_quantity = sqlx::query_scalar!(
        "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand + $1 WHERE id = $2 RETURNING quantity_on_hand",
        quantity_change,
        payload.item_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update inventory quantity: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Log audit trail
    state.log_inventory_audit(
        &mut tx,
        payload.item_id,
        &format!("TRANSACTION_{}", payload.transaction_type),
        Some(item.quantity_on_hand),
        new_quantity,
        user_id,
    ).await.map_err(|e| {
        error!("Failed to log audit trail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created {} transaction for item {} quantity {} by user {}", 
        payload.transaction_type, item.item_code, payload.quantity, user_id);

    let transaction_with_details = InventoryTransaction {
        item_code: Some(item.item_code),
        item_name: Some(item.item_name),
        ..transaction
    };

    Ok(Json(transaction_with_details))
}

async fn get_inventory_transactions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<InventoryTransaction>>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    
    let item_id_filter = params.get("item_id")
        .and_then(|id| Uuid::parse_str(id).ok());
    
    let limit: i64 = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50)
        .min(1000);

    let offset: i64 = params.get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    let transactions = if let Some(item_id) = item_id_filter {
        sqlx::query!(
            r#"
            SELECT it.id, it.company_id, it.item_id, it.transaction_type, it.transaction_date,
                   it.quantity, it.unit_cost, it.total_cost, it.reference, it.journal_entry_id,
                   it.created_at, i.item_code, i.item_name
            FROM inventory_transactions it
            LEFT JOIN inventory_items i ON it.item_id = i.id
            WHERE it.company_id = $1 AND it.item_id = $2
            ORDER BY it.transaction_date DESC, it.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            company_id,
            item_id,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query!(
            r#"
            SELECT it.id, it.company_id, it.item_id, it.transaction_type, it.transaction_date,
                   it.quantity, it.unit_cost, it.total_cost, it.reference, it.journal_entry_id,
                   it.created_at, i.item_code, i.item_name
            FROM inventory_transactions it
            LEFT JOIN inventory_items i ON it.item_id = i.id
            WHERE it.company_id = $1
            ORDER BY it.transaction_date DESC, it.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            company_id,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await
    };

    let transactions = transactions.map_err(|e| {
        error!("Failed to fetch inventory transactions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|row| InventoryTransaction {
        id: row.id,
        company_id: row.company_id,
        item_id: row.item_id,
        item_code: row.item_code,
        item_name: row.item_name,
        transaction_type: row.transaction_type,
        transaction_date: row.transaction_date,
        quantity: row.quantity,
        unit_cost: row.unit_cost,
        total_cost: row.total_cost,
        reference: row.reference,
        journal_entry_id: row.journal_entry_id,
        created_at: row.created_at,
    })
    .collect();

    Ok(Json(transactions))
}

async fn adjust_stock(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<StockAdjustmentRequest>,
) -> Result<Json<InventoryTransaction>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;

    if payload.adjustment_quantity == Decimal::ZERO {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get current item details
    let item = sqlx::query!(
        r#"
        SELECT item_code, item_name, quantity_on_hand, unit_cost
        FROM inventory_items 
        WHERE id = $1 AND company_id = $2 AND is_active = true
        "#,
        payload.item_id,
        company_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to fetch item details: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Check for sufficient inventory on negative adjustments
    let new_quantity = item.quantity_on_hand + payload.adjustment_quantity;
    if new_quantity < Decimal::ZERO {
        warn!("Stock adjustment would result in negative inventory for item {}: current {}, adjustment {}", 
            item.item_code, item.quantity_on_hand, payload.adjustment_quantity);
        return Err(StatusCode::CONFLICT);
    }

    let transaction_id = Uuid::new_v4();
    let transaction_type = if payload.adjustment_quantity > Decimal::ZERO { "IN" } else { "OUT" };
    let abs_quantity = payload.adjustment_quantity.abs();
    let total_cost = abs_quantity * item.unit_cost;

    // Create adjustment transaction
    let transaction = sqlx::query_as!(
        InventoryTransaction,
        r#"
        INSERT INTO inventory_transactions 
        (id, company_id, item_id, transaction_type, transaction_date, quantity, unit_cost, total_cost, reference)
        VALUES ($1, $2, $3, $4, CURRENT_DATE, $5, $6, $7, $8)
        RETURNING id, company_id, item_id, transaction_type, transaction_date, 
                  quantity, unit_cost, total_cost, reference, journal_entry_id, created_at
        "#,
        transaction_id,
        company_id,
        payload.item_id,
        transaction_type,
        abs_quantity,
        item.unit_cost,
        total_cost,
        Some(format!("ADJUSTMENT: {}", payload.reason))
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to create adjustment transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update inventory quantity
    sqlx::query!(
        "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand + $1 WHERE id = $2",
        payload.adjustment_quantity,
        payload.item_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update inventory quantity: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Log audit trail
    state.log_inventory_audit(
        &mut tx,
        payload.item_id,
        "STOCK_ADJUSTMENT",
        Some(item.quantity_on_hand),
        Some(new_quantity),
        user_id,
    ).await.map_err(|e| {
        error!("Failed to log audit trail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Stock adjustment for item {} by {} by user {}", 
        item.item_code, payload.adjustment_quantity, user_id);

    let transaction_with_details = InventoryTransaction {
        item_code: Some(item.item_code),
        item_name: Some(item.item_name),
        ..transaction
    };

    Ok(Json(transaction_with_details))
}

async fn get_stock_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let company_id = extract_company_id(&headers)?;

    let include_zero_stock = params.get("include_zero")
        .map(|v| v == "true")
        .unwrap_or(true);

    let stock_data = sqlx::query!(
        r#"
        SELECT 
            i.id,
            i.item_code,
            i.item_name,
            i.item_type::text,
            i.unit_of_measure,
            i.quantity_on_hand,
            i.reorder_level,
            i.unit_cost,
            i.selling_price,
            i.quantity_on_hand * i.unit_cost as stock_value,
            CASE 
                WHEN i.quantity_on_hand = 0 THEN 'OUT_OF_STOCK'
                WHEN i.quantity_on_hand <= i.reorder_level THEN 'LOW_STOCK'
                ELSE 'NORMAL'
            END as stock_status,
            i.created_at,
            i.updated_at
        FROM inventory_items i
        WHERE i.company_id = $1 AND i.is_active = true
        ORDER BY 
            CASE 
                WHEN i.quantity_on_hand = 0 THEN 1
                WHEN i.quantity_on_hand <= i.reorder_level THEN 2
                ELSE 3
            END,
            i.item_code
        "#,
        company_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch stock report data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let filtered_data: Vec<_> = if include_zero_stock {
        stock_data
    } else {
        stock_data.into_iter()
            .filter(|row| row.quantity_on_hand > Decimal::ZERO)
            .collect()
    };

    let total_value: Decimal = filtered_data.iter()
        .map(|row| row.stock_value.unwrap_or(Decimal::ZERO))
        .sum();

    let total_items = filtered_data.len();
    let out_of_stock_count = filtered_data.iter()
        .filter(|row| row.stock_status.as_ref().map(|s| s.as_str()) == Some("OUT_OF_STOCK"))
        .count();
    let low_stock_count = filtered_data.iter()
        .filter(|row| row.stock_status.as_ref().map(|s| s.as_str()) == Some("LOW_STOCK"))
        .count();

    // Get low stock alerts
    let low_stock_alerts: Vec<LowStockAlert> = filtered_data.iter()
        .filter_map(|row| {
            if row.stock_status.as_ref().map(|s| s.as_str()) == Some("LOW_STOCK") ||
               row.stock_status.as_ref().map(|s| s.as_str()) == Some("OUT_OF_STOCK") {
                Some(LowStockAlert {
                    item_id: row.id,
                    item_code: row.item_code.clone(),
                    item_name: row.item_name.clone(),
                    current_quantity: row.quantity_on_hand,
                    reorder_level: row.reorder_level,
                    shortage_amount: (row.reorder_level - row.quantity_on_hand).max(Decimal::ZERO),
                })
            } else {
                None
            }
        })
        .collect();

    let report = serde_json::json!({
        "company_id": company_id,
        "report_date": chrono::Utc::now().date_naive(),
        "summary": {
            "total_items": total_items,
            "total_stock_value": total_value,
            "out_of_stock_items": out_of_stock_count,
            "low_stock_items": low_stock_count,
            "normal_stock_items": total_items - out_of_stock_count - low_stock_count
        },
        "alerts": {
            "low_stock_alerts": low_stock_alerts,
            "reorder_suggestions": filtered_data.iter()
                .filter(|row| row.stock_status.as_ref().map(|s| s.as_str()) == Some("LOW_STOCK"))
                .map(|row| serde_json::json!({
                    "item_id": row.id,
                    "item_code": row.item_code,
                    "item_name": row.item_name,
                    "suggested_order_quantity": row.reorder_level * 2, // Simple reorder logic
                    "estimated_cost": row.reorder_level * 2 * row.unit_cost
                }))
                .collect::<Vec<_>>()
        },
        "items": filtered_data.iter().map(|row| serde_json::json!({
            "id": row.id,
            "item_code": row.item_code,
            "item_name": row.item_name,
            "item_type": row.item_type,
            "unit_of_measure": row.unit_of_measure,
            "quantity_on_hand": row.quantity_on_hand,
            "reorder_level": row.reorder_level,
            "unit_cost": row.unit_cost,
            "selling_price": row.selling_price,
            "stock_value": row.stock_value,
            "stock_status": row.stock_status,
            "last_updated": row.updated_at
        })).collect::<Vec<_>>()
    });

    Ok(Json(report))
}

async fn get_valuation_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let company_id = extract_company_id(&headers)?;
    
    let valuation_method = params.get("method").unwrap_or("AVERAGE_COST");

    let valuation_data = sqlx::query!(
        r#"
        SELECT 
            i.item_code,
            i.item_name,
            i.item_type::text,
            i.quantity_on_hand,
            i.unit_cost as current_unit_cost,
            COALESCE(
                (SELECT AVG(it.unit_cost) 
                 FROM inventory_transactions it 
                 WHERE it.item_id = i.id AND it.transaction_type = 'IN' 
                 AND it.transaction_date >= CURRENT_DATE - INTERVAL '90 days'),
                i.unit_cost
            ) as average_cost_90_days,
            COALESCE(
                (SELECT it.unit_cost 
                 FROM inventory_transactions it 
                 WHERE it.item_id = i.id AND it.transaction_type = 'IN' 
                 ORDER BY it.transaction_date DESC, it.created_at DESC 
                 LIMIT 1),
                i.unit_cost
            ) as last_purchase_cost
        FROM inventory_items i
        WHERE i.company_id = $1 AND i.is_active = true AND i.quantity_on_hand > 0
        ORDER BY i.item_code
        "#,
        company_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch valuation data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut total_current_value = Decimal::ZERO;
    let mut total_average_value = Decimal::ZERO;
    let mut total_fifo_value = Decimal::ZERO;

    let items_valuation: Vec<_> = valuation_data.iter().map(|row| {
        let current_value = row.quantity_on_hand * row.current_unit_cost;
        let average_value = row.quantity_on_hand * row.average_cost_90_days.unwrap_or(row.current_unit_cost);
        let fifo_value = row.quantity_on_hand * row.last_purchase_cost.unwrap_or(row.current_unit_cost);

        total_current_value += current_value;
        total_average_value += average_value;
        total_fifo_value += fifo_value;

        serde_json::json!({
            "item_code": row.item_code,
            "item_name": row.item_name,
            "item_type": row.item_type,
            "quantity_on_hand": row.quantity_on_hand,
            "valuation": {
                "current_cost": {
                    "unit_cost": row.current_unit_cost,
                    "total_value": current_value
                },
                "average_cost": {
                    "unit_cost": row.average_cost_90_days.unwrap_or(row.current_unit_cost),
                    "total_value": average_value
                },
                "fifo_cost": {
                    "unit_cost": row.last_purchase_cost.unwrap_or(row.current_unit_cost),
                    "total_value": fifo_value
                }
            }
        })
    }).collect();

    let report = serde_json::json!({
        "company_id": company_id,
        "valuation_date": chrono::Utc::now().date_naive(),
        "method": valuation_method,
        "summary": {
            "total_items": valuation_data.len(),
            "total_current_value": total_current_value,
            "total_average_value": total_average_value,
            "total_fifo_value": total_fifo_value,
            "valuation_difference": {
                "average_vs_current": total_average_value - total_current_value,
                "fifo_vs_current": total_fifo_value - total_current_value
            }
        },
        "items": items_valuation
    });

    Ok(Json(report))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,inventory_service=debug")
        .init();

    info!("Starting Inventory Management Service...");

    // Create the database pool using our custom function
    let pool = create_inventory_database_pool().await?;

    let app_state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/items", post(create_inventory_item))
        .route("/items", get(get_inventory_items))
        .route("/items/:id", get(get_inventory_item))
        .route("/items/:id", put(update_inventory_item))
        .route("/transactions", post(create_inventory_transaction))
        .route("/transactions", get(get_inventory_transactions))
        .route("/stock-adjustment", post(adjust_stock))
        .route("/stock-report", get(get_stock_report))
        .route("/valuation-report", get(get_valuation_report))
        .with_state(app_state);

    let bind_addr = env::var("INVENTORY_MANAGEMENT_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3008".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Inventory Management service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}