use crate::models::*;
use common::{ServiceResult, ServiceError, PaginationParams};
use rust_decimal::Decimal;
use sqlx::{PgPool, Transaction, Postgres};
use uuid::Uuid;

pub struct CustomerService {
    db: PgPool,
    audit_logger: database::audit::AuditLogger,
}

impl CustomerService {
    pub fn new(db: PgPool) -> Self {
        let audit_logger = database::audit::AuditLogger::new(db.clone());
        Self { db, audit_logger }
    }

    pub async fn create_customer(
        &self,
        mut request: CreateCustomerRequest,
        company_id: Uuid,
        user_id: Uuid,
    ) -> ServiceResult<Customer> {
        request.company_id = company_id;
        
        // Validate customer code uniqueness
        let existing_customer = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM customers WHERE company_id = $1 AND customer_code = $2)",
            company_id,
            request.customer_code
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .unwrap_or(false);

        if existing_customer {
            return Err(ServiceError::Conflict(
                format!("Customer code '{}' already exists", request.customer_code)
            ));
        }

        // Validate NPWP if provided
        if let Some(ref npwp) = request.npwp {
            if !self.validate_npwp(npwp) {
                return Err(ServiceError::Validation("Invalid NPWP format".to_string()));
            }
        }
        
        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;
        let customer_id = Uuid::new_v4();
        
        let customer = sqlx::query_as!(
            Customer,
            r#"
            INSERT INTO customers (id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())
            RETURNING id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
            "#,
            customer_id,
            company_id,
            request.customer_code,
            request.customer_name,
            request.npwp,
            request.address,
            request.phone,
            request.email,
            request.credit_limit.unwrap_or(Decimal::ZERO),
            request.payment_terms.unwrap_or(30)
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Log audit trail
        self.audit_logger.log_activity(
            &mut tx,
            "customers",
            customer_id,
            "CREATE",
            None,
            Some(serde_json::to_value(&customer).unwrap()),
            user_id,
        ).await.map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Created customer {} for company {}", customer.customer_code, company_id);

        Ok(customer)
    }

    pub async fn get_customers(
        &self,
        company_id: Uuid,
        include_inactive: bool,
        search_term: Option<&String>,
        pagination: PaginationParams,
    ) -> ServiceResult<Vec<Customer>> {
        let limit = pagination.limit();
        let offset = pagination.offset();

        let customers = match (include_inactive, search_term) {
            (true, Some(search)) => {
                sqlx::query_as!(
                    Customer,
                    r#"
                    SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                    FROM customers 
                    WHERE company_id = $1 
                      AND (customer_name ILIKE $2 OR customer_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                    ORDER BY customer_name
                    LIMIT $3 OFFSET $4
                    "#,
                    company_id,
                    format!("%{}%", search),
                    limit,
                    offset
                )
                .fetch_all(&self.db)
                .await
            }
            (false, Some(search)) => {
                sqlx::query_as!(
                    Customer,
                    r#"
                    SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                    FROM customers 
                    WHERE company_id = $1 AND is_active = true
                      AND (customer_name ILIKE $2 OR customer_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                    ORDER BY customer_name
                    LIMIT $3 OFFSET $4
                    "#,
                    company_id,
                    format!("%{}%", search),
                    limit,
                    offset
                )
                .fetch_all(&self.db)
                .await
            }
            (true, None) => {
                sqlx::query_as!(
                    Customer,
                    r#"
                    SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                    FROM customers 
                    WHERE company_id = $1
                    ORDER BY customer_name
                    LIMIT $2 OFFSET $3
                    "#,
                    company_id,
                    limit,
                    offset
                )
                .fetch_all(&self.db)
                .await
            }
            (false, None) => {
                sqlx::query_as!(
                    Customer,
                    r#"
                    SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
                    FROM customers 
                    WHERE company_id = $1 AND is_active = true
                    ORDER BY customer_name
                    LIMIT $2 OFFSET $3
                    "#,
                    company_id,
                    limit,
                    offset
                )
                .fetch_all(&self.db)
                .await
            }
        };

        customers.map_err(ServiceError::Database)
    }

    pub async fn get_customer_by_id(
        &self,
        customer_id: Uuid,
        company_id: Uuid,
    ) -> ServiceResult<Customer> {
        let customer = sqlx::query_as!(
            Customer,
            r#"
            SELECT id, company_id, customer_code, customer_name, npwp, address, phone, email, credit_limit, payment_terms, is_active, created_at, updated_at
            FROM customers 
            WHERE id = $1 AND company_id = $2
            "#,
            customer_id,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Customer not found".to_string()))?;

        Ok(customer)
    }

    pub async fn check_credit_limit(
        &self,
        customer_id: Uuid,
        company_id: Uuid,
        additional_amount: Decimal,
    ) -> ServiceResult<bool> {
        let result = sqlx::query!(
            r#"
            SELECT 
                c.credit_limit,
                COALESCE(SUM(ci.total_amount - ci.paid_amount), 0) as current_outstanding
            FROM customers c
            LEFT JOIN customer_invoices ci ON c.id = ci.customer_id 
                AND ci.status != 'CANCELLED' AND ci.total_amount > ci.paid_amount
            WHERE c.id = $1 AND c.company_id = $2
            GROUP BY c.id, c.credit_limit
            "#,
            customer_id,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        if let Some(row) = result {
            let current_outstanding = row.current_outstanding.unwrap_or(Decimal::ZERO);
            let available_credit = row.credit_limit - current_outstanding;
            Ok(available_credit >= additional_amount)
        } else {
            Err(ServiceError::NotFound("Customer not found".to_string()))
        }
    }

    pub async fn get_customer_credit_info(
        &self,
        customer_id: Uuid,
        company_id: Uuid,
    ) -> ServiceResult<CustomerCreditInfo> {
        let result = sqlx::query!(
            r#"
            SELECT 
                c.credit_limit,
                c.customer_name,
                COALESCE(SUM(ci.total_amount - ci.paid_amount), 0) as current_outstanding,
                COUNT(ci.id) FILTER (WHERE ci.status != 'CANCELLED' AND ci.total_amount > ci.paid_amount) as outstanding_invoices,
                MAX(ci.due_date) FILTER (WHERE ci.status != 'PAID' AND ci.status != 'CANCELLED') as oldest_due_date
            FROM customers c
            LEFT JOIN customer_invoices ci ON c.id = ci.customer_id
            WHERE c.id = $1 AND c.company_id = $2
            GROUP BY c.id, c.credit_limit, c.customer_name
            "#,
            customer_id,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Customer not found".to_string()))?;

        let current_outstanding = result.current_outstanding.unwrap_or(Decimal::ZERO);
        let available_credit = result.credit_limit - current_outstanding;
        let credit_utilization = if result.credit_limit > Decimal::ZERO {
            (current_outstanding / result.credit_limit * Decimal::new(100, 0)).to_string().parse().unwrap_or(0.0)
        } else {
            0.0
        };

        let days_past_due = if let Some(oldest_due) = result.oldest_due_date {
            let today = chrono::Utc::now().date_naive();
            if oldest_due < today {
                Some((today - oldest_due).num_days() as i32)
            } else {
                None
            }
        } else {
            None
        };

        Ok(CustomerCreditInfo {
            customer_id,
            customer_name: result.customer_name,
            credit_limit: result.credit_limit,
            current_outstanding,
            available_credit,
            credit_utilization,
            outstanding_invoices: result.outstanding_invoices.unwrap_or(0) as u32,
            days_past_due,
            credit_status: if available_credit < Decimal::ZERO {
                "OVER_LIMIT".to_string()
            } else if credit_utilization > 80.0 {
                "HIGH_UTILIZATION".to_string()
            } else {
                "GOOD".to_string()
            },
        })
    }

    fn validate_npwp(&self, npwp: &str) -> bool {
        // NPWP format: XX.XXX.XXX.X-XXX.XXX (15 digits)
        let clean_npwp: String = npwp.chars().filter(|c| c.is_ascii_digit()).collect();
        clean_npwp.len() == 15
    }
}