use crate::models::*;
use common::{ServiceResult, ServiceError, PaginationParams};
use sqlx::{PgPool, Transaction, Postgres};
use uuid::Uuid;

pub struct VendorService {
    db: PgPool,
    audit_logger: database::audit::AuditLogger,
}

impl VendorService {
    pub fn new(db: PgPool) -> Self {
        let audit_logger = database::audit::AuditLogger::new(db.clone());
        Self { db, audit_logger }
    }

    pub async fn create_vendor(
        &self,
        mut request: CreateVendorRequest,
        company_id: Uuid,
        user_id: Uuid,
    ) -> ServiceResult<Vendor> {
        request.company_id = company_id;
        
        // Validate vendor code uniqueness
        let existing_vendor = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM vendors WHERE company_id = $1 AND vendor_code = $2)",
            company_id,
            request.vendor_code
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .unwrap_or(false);

        if existing_vendor {
            return Err(ServiceError::Conflict(
                format!("Vendor code '{}' already exists", request.vendor_code)
            ));
        }

        // Validate NPWP if provided
        if let Some(ref npwp) = request.npwp {
            if !self.validate_npwp(npwp) {
                return Err(ServiceError::Validation("Invalid NPWP format".to_string()));
            }
        }
        
        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;
        let vendor_id = Uuid::new_v4();
        
        let vendor = sqlx::query_as!(
            Vendor,
            r#"
            INSERT INTO vendors (id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, NOW(), NOW())
            RETURNING id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
            "#,
            vendor_id,
            company_id,
            request.vendor_code,
            request.vendor_name,
            request.npwp,
            request.address,
            request.phone,
            request.email,
            request.payment_terms.unwrap_or(30)
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Log audit trail
        self.audit_logger.log_activity(
            &mut tx,
            "vendors",
            vendor_id,
            "CREATE",
            None,
            Some(serde_json::to_value(&vendor).unwrap()),
            user_id,
        ).await.map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Created vendor {} for company {}", vendor.vendor_code, company_id);

        Ok(vendor)
    }

    pub async fn get_vendors(
        &self,
        company_id: Uuid,
        include_inactive: bool,
        search_term: Option<&String>,
        pagination: PaginationParams,
    ) -> ServiceResult<Vec<Vendor>> {
        let limit = pagination.limit();
        let offset = pagination.offset();

        let vendors = match (include_inactive, search_term) {
            (true, Some(search)) => {
                sqlx::query_as!(
                    Vendor,
                    r#"
                    SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                    FROM vendors 
                    WHERE company_id = $1 
                      AND (vendor_name ILIKE $2 OR vendor_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                    ORDER BY vendor_name
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
                    Vendor,
                    r#"
                    SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                    FROM vendors 
                    WHERE company_id = $1 AND is_active = true
                      AND (vendor_name ILIKE $2 OR vendor_code ILIKE $2 OR COALESCE(npwp, '') ILIKE $2)
                    ORDER BY vendor_name
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
                    Vendor,
                    r#"
                    SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                    FROM vendors 
                    WHERE company_id = $1
                    ORDER BY vendor_name
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
                    Vendor,
                    r#"
                    SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
                    FROM vendors 
                    WHERE company_id = $1 AND is_active = true
                    ORDER BY vendor_name
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

        vendors.map_err(ServiceError::Database)
    }

    pub async fn get_vendor_by_id(
        &self,
        vendor_id: Uuid,
        company_id: Uuid,
    ) -> ServiceResult<Vendor> {
        let vendor = sqlx::query_as!(
            Vendor,
            r#"
            SELECT id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
            FROM vendors 
            WHERE id = $1 AND company_id = $2
            "#,
            vendor_id,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Vendor not found".to_string()))?;

        Ok(vendor)
    }

    pub async fn update_vendor(
        &self,
        vendor_id: Uuid,
        request: UpdateVendorRequest,
        company_id: Uuid,
        user_id: Uuid,
    ) -> ServiceResult<Vendor> {
        // Validate NPWP if provided
        if let Some(ref npwp) = request.npwp {
            if !self.validate_npwp(npwp) {
                return Err(ServiceError::Validation("Invalid NPWP format".to_string()));
            }
        }

        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        // Get current vendor for audit log
        let old_vendor = self.get_vendor_by_id(vendor_id, company_id).await?;

        let vendor = sqlx::query_as!(
            Vendor,
            r#"
            UPDATE vendors 
            SET vendor_name = $1, npwp = $2, address = $3, phone = $4, email = $5, 
                payment_terms = $6, is_active = $7, updated_at = NOW()
            WHERE id = $8 AND company_id = $9
            RETURNING id, company_id, vendor_code, vendor_name, npwp, address, phone, email, payment_terms, is_active, created_at, updated_at
            "#,
            request.vendor_name,
            request.npwp,
            request.address,
            request.phone,
            request.email,
            request.payment_terms,
            request.is_active,
            vendor_id,
            company_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Vendor not found".to_string()))?;

        // Log audit trail
        self.audit_logger.log_activity(
            &mut tx,
            "vendors",
            vendor_id,
            "UPDATE",
            Some(serde_json::to_value(&old_vendor).unwrap()),
            Some(serde_json::to_value(&vendor).unwrap()),
            user_id,
        ).await.map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Updated vendor {} for company {}", vendor.vendor_code, company_id);

        Ok(vendor)
    }

    pub async fn delete_vendor(
        &self,
        vendor_id: Uuid,
        company_id: Uuid,
        user_id: Uuid,
    ) -> ServiceResult<()> {
        // Check if vendor has associated invoices
        let has_invoices = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM vendor_invoices WHERE vendor_id = $1)",
            vendor_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .unwrap_or(false);

        if has_invoices {
            return Err(ServiceError::Conflict(
                "Cannot delete vendor with associated invoices. Deactivate instead.".to_string()
            ));
        }

        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        // Get vendor for audit log
        let vendor = self.get_vendor_by_id(vendor_id, company_id).await?;

        let deleted_count = sqlx::query!(
            "DELETE FROM vendors WHERE id = $1 AND company_id = $2",
            vendor_id,
            company_id
        )
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .rows_affected();

        if deleted_count == 0 {
            return Err(ServiceError::NotFound("Vendor not found".to_string()));
        }

        // Log audit trail
        self.audit_logger.log_activity(
            &mut tx,
            "vendors",
            vendor_id,
            "DELETE",
            Some(serde_json::to_value(&vendor).unwrap()),
            None,
            user_id,
        ).await.map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Deleted vendor {} for company {}", vendor.vendor_code, company_id);

        Ok(())
    }

    pub async fn get_vendor_statistics(
        &self,
        vendor_id: Uuid,
        company_id: Uuid,
    ) -> ServiceResult<VendorStatistics> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(vi.id) as total_invoices,
                COALESCE(SUM(vi.total_amount), 0) as total_amount,
                COALESCE(SUM(vi.paid_amount), 0) as paid_amount,
                COALESCE(SUM(vi.total_amount - vi.paid_amount), 0) as outstanding_amount,
                COALESCE(AVG(vi.total_amount), 0) as average_invoice_amount
            FROM vendor_invoices vi
            WHERE vi.vendor_id = $1 AND vi.company_id = $2
            "#,
            vendor_id,
            company_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        Ok(VendorStatistics {
            vendor_id,
            total_invoices: stats.total_invoices.unwrap_or(0) as u32,
            total_amount: stats.total_amount.unwrap_or_default(),
            paid_amount: stats.paid_amount.unwrap_or_default(),
            outstanding_amount: stats.outstanding_amount.unwrap_or_default(),
            average_invoice_amount: stats.average_invoice_amount.unwrap_or_default(),
        })
    }

    fn validate_npwp(&self, npwp: &str) -> bool {
        // NPWP format: XX.XXX.XXX.X-XXX.XXX (15 digits)
        let clean_npwp: String = npwp.chars().filter(|c| c.is_ascii_digit()).collect();
        clean_npwp.len() == 15
    }
}