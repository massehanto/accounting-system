use crate::models::*;
use common::{ServiceResult, ServiceError};
use rust_decimal::Decimal;
use sqlx::{PgPool, Transaction, Postgres};
use uuid::Uuid;

pub struct InventoryService {
    db: PgPool,
}

impl InventoryService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_item(
        &self,
        request: CreateInventoryItemRequest,
        user_id: Uuid,
    ) -> ServiceResult<InventoryItem> {
        // Validate item code uniqueness
        let existing_item = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM inventory_items WHERE company_id = $1 AND item_code = $2)",
            request.company_id,
            request.item_code
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .unwrap_or(false);

        if existing_item {
            return Err(ServiceError::Conflict(
                format!("Item code '{}' already exists", request.item_code)
            ));
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
            request.company_id,
            request.item_code,
            request.item_name,
            request.description,
            request.item_type as ItemType,
            request.unit_of_measure,
            request.unit_cost,
            request.selling_price,
            request.reorder_level.unwrap_or(Decimal::ZERO)
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        tracing::info!("Created inventory item {} for company {} by user {}", 
            request.item_code, request.company_id, user_id);

        Ok(item)
    }

    pub async fn create_transaction(
        &self,
        request: CreateInventoryTransactionRequest,
        user_id: Uuid,
    ) -> ServiceResult<InventoryTransaction> {
        // Validate transaction type
        if !matches!(request.transaction_type.as_str(), "IN" | "OUT") {
            return Err(ServiceError::Validation(
                "Transaction type must be 'IN' or 'OUT'".to_string()
            ));
        }

        if request.quantity <= Decimal::ZERO {
            return Err(ServiceError::Validation(
                "Quantity must be positive".to_string()
            ));
        }

        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        // Get current item details and verify ownership
        let item = sqlx::query!(
            r#"
            SELECT item_code, item_name, quantity_on_hand 
            FROM inventory_items 
            WHERE id = $1 AND company_id = $2 AND is_active = true
            "#,
            request.item_id,
            request.company_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Item not found".to_string()))?;

        // Check for sufficient inventory on OUT transactions
        if request.transaction_type == "OUT" && item.quantity_on_hand < request.quantity {
            return Err(ServiceError::Conflict(
                format!("Insufficient inventory: requested {}, available {}", 
                    request.quantity, item.quantity_on_hand)
            ));
        }

        let transaction_id = Uuid::new_v4();
        let total_cost = request.quantity * request.unit_cost;

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
            request.company_id,
            request.item_id,
            request.transaction_type,
            request.transaction_date,
            request.quantity,
            request.unit_cost,
            total_cost,
            request.reference
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Update inventory quantity
        let quantity_change = if request.transaction_type == "IN" {
            request.quantity
        } else {
            -request.quantity
        };

        let new_quantity = sqlx::query_scalar!(
            "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand + $1, updated_at = NOW() WHERE id = $2 RETURNING quantity_on_hand",
            quantity_change,
            request.item_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Created {} transaction for item {} quantity {} by user {}", 
            request.transaction_type, item.item_code, request.quantity, user_id);

        let transaction_with_details = InventoryTransaction {
            item_code: Some(item.item_code),
            item_name: Some(item.item_name),
            ..transaction
        };

        Ok(transaction_with_details)
    }

    pub async fn adjust_stock(
        &self,
        request: StockAdjustmentRequest,
        company_id: Uuid,
        user_id: Uuid,
    ) -> ServiceResult<InventoryTransaction> {
        if request.adjustment_quantity == Decimal::ZERO {
            return Err(ServiceError::Validation(
                "Adjustment quantity cannot be zero".to_string()
            ));
        }

        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        // Get current item details
        let item = sqlx::query!(
            r#"
            SELECT item_code, item_name, quantity_on_hand, unit_cost
            FROM inventory_items 
            WHERE id = $1 AND company_id = $2 AND is_active = true
            "#,
            request.item_id,
            company_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Item not found".to_string()))?;

        // Check for sufficient inventory on negative adjustments
        let new_quantity = item.quantity_on_hand + request.adjustment_quantity;
        if new_quantity < Decimal::ZERO {
            return Err(ServiceError::Conflict(
                format!("Adjustment would result in negative inventory: current {}, adjustment {}", 
                    item.quantity_on_hand, request.adjustment_quantity)
            ));
        }

        let transaction_id = Uuid::new_v4();
        let transaction_type = if request.adjustment_quantity > Decimal::ZERO { "IN" } else { "OUT" };
        let abs_quantity = request.adjustment_quantity.abs();
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
            request.item_id,
            transaction_type,
            abs_quantity,
            item.unit_cost,
            total_cost,
            Some(format!("ADJUSTMENT: {}", request.reason))
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Update inventory quantity
        sqlx::query!(
            "UPDATE inventory_items SET quantity_on_hand = quantity_on_hand + $1, updated_at = NOW() WHERE id = $2",
            request.adjustment_quantity,
            request.item_id
        )
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Stock adjustment for item {} by {} by user {}", 
            item.item_code, request.adjustment_quantity, user_id);

        let transaction_with_details = InventoryTransaction {
            item_code: Some(item.item_code),
            item_name: Some(item.item_name),
            ..transaction
        };

        Ok(transaction_with_details)
    }

    pub async fn get_low_stock_alerts(&self, company_id: Uuid) -> ServiceResult<Vec<LowStockAlert>> {
        let alerts = sqlx::query!(
            r#"
            SELECT id, item_code, item_name, quantity_on_hand, reorder_level
            FROM inventory_items
            WHERE company_id = $1 AND is_active = true 
                  AND (quantity_on_hand <= reorder_level OR quantity_on_hand = 0)
            ORDER BY 
                CASE WHEN quantity_on_hand = 0 THEN 1 ELSE 2 END,
                (reorder_level - quantity_on_hand) DESC
            "#,
            company_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let low_stock_alerts: Vec<LowStockAlert> = alerts
            .into_iter()
            .map(|row| LowStockAlert {
                item_id: row.id,
                item_code: row.item_code,
                item_name: row.item_name,
                current_quantity: row.quantity_on_hand,
                reorder_level: row.reorder_level,
                shortage_amount: (row.reorder_level - row.quantity_on_hand).max(Decimal::ZERO),
            })
            .collect();

        Ok(low_stock_alerts)
    }

    pub async fn calculate_inventory_valuation(
        &self,
        company_id: Uuid,
        method: super::ValuationMethod,
    ) -> ServiceResult<InventoryValuation> {
        let items_data = sqlx::query!(
            r#"
            SELECT 
                i.id,
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
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut total_value = Decimal::ZERO;
        let mut items_valuation = Vec::new();

        for row in items_data {
            let unit_cost = match method {
                super::ValuationMethod::CurrentCost => row.current_unit_cost,
                super::ValuationMethod::AverageCost => row.average_cost_90_days.unwrap_or(row.current_unit_cost),
                super::ValuationMethod::Fifo => row.last_purchase_cost.unwrap_or(row.current_unit_cost),
            };

            let item_value = row.quantity_on_hand * unit_cost;
            total_value += item_value;

            items_valuation.push(ItemValuation {
                item_id: row.id,
                item_code: row.item_code,
                item_name: row.item_name,
                item_type: row.item_type.unwrap_or_default(),
                quantity_on_hand: row.quantity_on_hand,
                unit_cost,
                total_value: item_value,
            });
        }

        Ok(InventoryValuation {
            company_id,
            valuation_date: chrono::Utc::now().date_naive(),
            method,
            total_value,
            items: items_valuation,
        })
    }

    pub async fn get_stock_movements(
        &self,
        company_id: Uuid,
        item_id: Option<Uuid>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> ServiceResult<Vec<StockMovement>> {
        let movements = if let Some(item_id) = item_id {
            sqlx::query!(
                r#"
                SELECT 
                    it.id,
                    it.item_id,
                    i.item_code,
                    i.item_name,
                    it.transaction_type,
                    it.transaction_date,
                    it.quantity,
                    it.unit_cost,
                    it.reference,
                    it.created_at
                FROM inventory_transactions it
                JOIN inventory_items i ON it.item_id = i.id
                WHERE it.company_id = $1 AND it.item_id = $2
                      AND it.transaction_date >= $3 AND it.transaction_date <= $4
                ORDER BY it.transaction_date DESC, it.created_at DESC
                "#,
                company_id,
                item_id,
                start_date,
                end_date
            )
            .fetch_all(&self.db)
            .await
        } else {
            sqlx::query!(
                r#"
                SELECT 
                    it.id,
                    it.item_id,
                    i.item_code,
                    i.item_name,
                    it.transaction_type,
                    it.transaction_date,
                    it.quantity,
                    it.unit_cost,
                    it.reference,
                    it.created_at
                FROM inventory_transactions it
                JOIN inventory_items i ON it.item_id = i.id
                WHERE it.company_id = $1
                      AND it.transaction_date >= $2 AND it.transaction_date <= $3
                ORDER BY it.transaction_date DESC, it.created_at DESC
                "#,
                company_id,
                start_date,
                end_date
            )
            .fetch_all(&self.db)
            .await
        };

        let movements = movements.map_err(ServiceError::Database)?
            .into_iter()
            .map(|row| StockMovement {
                transaction_id: row.id,
                item_id: row.item_id,
                item_code: row.item_code,
                item_name: row.item_name,
                transaction_type: row.transaction_type,
                transaction_date: row.transaction_date,
                quantity: row.quantity,
                unit_cost: row.unit_cost,
                reference: row.reference,
                created_at: row.created_at,
            })
            .collect();

        Ok(movements)
    }
}