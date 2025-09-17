use crate::models::*;
use common::{ServiceResult, ServiceError};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CompanyService {
    db: PgPool,
}

impl CompanyService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_company(
        &self,
        request: CreateCompanyRequest,
        user_id: Uuid,
    ) -> ServiceResult<Company> {
        // Validate NPWP format (Indonesian tax number)
        if !self.validate_npwp(&request.npwp) {
            return Err(ServiceError::Validation(
                "Invalid NPWP format".to_string()
            ));
        }

        // Check if NPWP already exists
        let existing_company = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM companies WHERE npwp = $1)",
            request.npwp
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .unwrap_or(false);

        if existing_company {
            return Err(ServiceError::Conflict(
                "Company with this NPWP already exists".to_string()
            ));
        }
        
        let company_id = Uuid::new_v4();
        
        let company = sqlx::query_as!(
            Company,
            r#"
            INSERT INTO companies (id, name, npwp, address, phone, email, business_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING id, name, npwp, address, phone, email, business_type, created_at, updated_at
            "#,
            company_id,
            request.name,
            request.npwp,
            request.address,
            request.phone,
            request.email,
            request.business_type
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        // Create default company settings
        self.create_default_settings(company_id).await?;

        tracing::info!("Created company {} ({}) by user {}", 
            company.name, company.id, user_id);

        Ok(company)
    }

    pub async fn update_company(
        &self,
        company_id: Uuid,
        request: UpdateCompanyRequest,
        requesting_user_company_id: Uuid,
    ) -> ServiceResult<Company> {
        // Ensure user can only update their own company
        if company_id != requesting_user_company_id {
            return Err(ServiceError::Authorization(
                "Cannot update another company's information".to_string()
            ));
        }

        let company = sqlx::query_as!(
            Company,
            r#"
            UPDATE companies 
            SET name = $1, address = $2, phone = $3, email = $4, business_type = $5, updated_at = NOW()
            WHERE id = $6
            RETURNING id, name, npwp, address, phone, email, business_type, created_at, updated_at
            "#,
            request.name,
            request.address,
            request.phone,
            request.email,
            request.business_type,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Company not found".to_string()))?;

        tracing::info!("Updated company {} ({})", company.name, company.id);

        Ok(company)
    }

    pub async fn get_company_settings(
        &self,
        company_id: Uuid,
    ) -> ServiceResult<CompanySettings> {
        let settings = sqlx::query_as!(
            CompanySettings,
            r#"
            SELECT 
                company_id,
                fiscal_year_start,
                default_currency,
                timezone,
                date_format,
                number_format,
                tax_settings,
                accounting_method,
                enable_multi_currency,
                auto_backup,
                notification_settings,
                created_at,
                updated_at
            FROM company_settings 
            WHERE company_id = $1
            "#,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Company settings not found".to_string()))?;

        Ok(settings)
    }

    pub async fn update_company_settings(
        &self,
        company_id: Uuid,
        request: UpdateCompanySettingsRequest,
        requesting_user_company_id: Uuid,
    ) -> ServiceResult<CompanySettings> {
        // Ensure user can only update their own company settings
        if company_id != requesting_user_company_id {
            return Err(ServiceError::Authorization(
                "Cannot update another company's settings".to_string()
            ));
        }

        let settings = sqlx::query_as!(
            CompanySettings,
            r#"
            UPDATE company_settings 
            SET 
                fiscal_year_start = $1,
                default_currency = $2,
                timezone = $3,
                date_format = $4,
                number_format = $5,
                tax_settings = $6,
                accounting_method = $7,
                enable_multi_currency = $8,
                auto_backup = $9,
                notification_settings = $10,
                updated_at = NOW()
            WHERE company_id = $11
            RETURNING 
                company_id,
                fiscal_year_start,
                default_currency,
                timezone,
                date_format,
                number_format,
                tax_settings,
                accounting_method,
                enable_multi_currency,
                auto_backup,
                notification_settings,
                created_at,
                updated_at
            "#,
            request.fiscal_year_start,
            request.default_currency,
            request.timezone,
            request.date_format,
            request.number_format,
            request.tax_settings,
            request.accounting_method,
            request.enable_multi_currency,
            request.auto_backup,
            request.notification_settings,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Company settings not found".to_string()))?;

        tracing::info!("Updated settings for company {}", company_id);

        Ok(settings)
    }

    async fn create_default_settings(&self, company_id: Uuid) -> ServiceResult<()> {
        let default_tax_settings = serde_json::json!({
            "ppn_rate": 11.0,
            "pph21_rate": 5.0,
            "pph22_rate": 1.5,
            "pph23_rate": 2.0,
            "enable_e_faktur": false,
            "enable_e_spt": false
        });

        let default_notification_settings = serde_json::json!({
            "email_notifications": true,
            "low_stock_alerts": true,
            "payment_reminders": true,
            "monthly_reports": true
        });

        sqlx::query!(
            r#"
            INSERT INTO company_settings (
                company_id, fiscal_year_start, default_currency, timezone, 
                date_format, number_format, tax_settings, accounting_method,
                enable_multi_currency, auto_backup, notification_settings,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            "#,
            company_id,
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), // Default fiscal year start
            "IDR",                          // Indonesian Rupiah
            "Asia/Jakarta",                 // Indonesian timezone
            "DD/MM/YYYY",                   // Indonesian date format
            "1,234.56",                     // Number format
            default_tax_settings,
            "ACCRUAL",                      // Default accounting method
            false,                          // Multi-currency disabled by default
            true,                           // Auto backup enabled
            default_notification_settings,
        )
        .execute(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        Ok(())
    }

    fn validate_npwp(&self, npwp: &str) -> bool {
        // NPWP format: XX.XXX.XXX.X-XXX.XXX (15 digits)
        let clean_npwp: String = npwp.chars().filter(|c| c.is_ascii_digit()).collect();
        clean_npwp.len() == 15
    }

    pub async fn get_company_statistics(
        &self,
        company_id: Uuid,
    ) -> ServiceResult<CompanyStatistics> {
        // This would typically call other services to gather statistics
        // For now, return basic information
        let company = sqlx::query!(
            "SELECT name, created_at FROM companies WHERE id = $1",
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Company not found".to_string()))?;

        let stats = CompanyStatistics {
            company_id,
            company_name: company.name,
            registration_date: company.created_at.date_naive(),
            total_users: 0,           // Would query auth service
            total_transactions: 0,    // Would query general ledger
            total_customers: 0,       // Would query accounts receivable
            total_vendors: 0,         // Would query accounts payable
            total_inventory_items: 0, // Would query inventory service
            last_backup_date: None,   // Would check backup system
            subscription_status: "ACTIVE".to_string(), // Would check billing
        };

        Ok(stats)
    }
}