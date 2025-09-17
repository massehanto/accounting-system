use crate::models::*;
use common::{ServiceResult, ServiceError};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ChartService {
    db: PgPool,
}

impl ChartService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_account(
        &self,
        request: CreateAccountRequest,
        user_id: Uuid,
    ) -> ServiceResult<Account> {
        // Validate account code format
        self.validate_account_code(&request.account_code)?;

        // Check if account code already exists
        let existing_account = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE company_id = $1 AND account_code = $2)",
            request.company_id,
            request.account_code
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .unwrap_or(false);

        if existing_account {
            return Err(ServiceError::Conflict(
                format!("Account code '{}' already exists", request.account_code)
            ));
        }

        // Validate parent account if specified
        if let Some(parent_id) = request.parent_account_id {
            self.validate_parent_account(request.company_id, parent_id, &request.account_type).await?;
        }

        let account_id = Uuid::new_v4();
        
        let account = sqlx::query_as!(
            Account,
            r#"
            INSERT INTO accounts (id, company_id, account_code, account_name, account_type, account_subtype, parent_account_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
            RETURNING id, company_id, account_code, account_name, 
                      account_type as "account_type: AccountType", 
                      account_subtype as "account_subtype: Option<AccountSubtype>", 
                      parent_account_id, is_active, created_at, updated_at
            "#,
            account_id,
            request.company_id,
            request.account_code,
            request.account_name,
            request.account_type as AccountType,
            request.account_subtype as Option<AccountSubtype>,
            request.parent_account_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        tracing::info!("Created account {} - {} ({}) by user {}", 
            account.account_code, account.account_name, account.id, user_id);

        Ok(account)
    }

    pub async fn create_from_template(
        &self,
        company_id: Uuid,
        template_name: &str,
        user_id: Uuid,
    ) -> ServiceResult<Vec<Account>> {
        let template_accounts = self.get_template_accounts(template_name)?;
        let mut created_accounts = Vec::new();

        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        for template_account in template_accounts {
            let account_id = Uuid::new_v4();
            
            let account = sqlx::query_as!(
                Account,
                r#"
                INSERT INTO accounts (id, company_id, account_code, account_name, account_type, account_subtype, parent_account_id, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
                RETURNING id, company_id, account_code, account_name, 
                          account_type as "account_type: AccountType", 
                          account_subtype as "account_subtype: Option<AccountSubtype>", 
                          parent_account_id, is_active, created_at, updated_at
                "#,
                account_id,
                company_id,
                template_account.account_code,
                template_account.account_name,
                template_account.account_type as AccountType,
                template_account.account_subtype as Option<AccountSubtype>,
                template_account.parent_account_id
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(ServiceError::Database)?;

            created_accounts.push(account);
        }

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Created {} accounts from template '{}' for company {} by user {}", 
            created_accounts.len(), template_name, company_id, user_id);

        Ok(created_accounts)
    }

    pub async fn validate_account_structure(
        &self,
        company_id: Uuid,
    ) -> ServiceResult<AccountValidationResult> {
        let accounts = sqlx::query_as!(
            Account,
            r#"
            SELECT id, company_id, account_code, account_name,
                   account_type as "account_type: AccountType",
                   account_subtype as "account_subtype: Option<AccountSubtype>",
                   parent_account_id, is_active, created_at, updated_at
            FROM accounts 
            WHERE company_id = $1 AND is_active = true
            ORDER BY account_code
            "#,
            company_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut validation_result = AccountValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Check for required account types
        let required_types = vec![
            AccountType::Asset,
            AccountType::Liability,
            AccountType::Equity,
            AccountType::Revenue,
            AccountType::Expense,
        ];

        for required_type in required_types {
            let has_type = accounts.iter().any(|a| a.account_type == required_type);
            if !has_type {
                validation_result.is_valid = false;
                validation_result.errors.push(
                    format!("Missing required account type: {:?}", required_type)
                );
            }
        }

        // Check for orphaned accounts (parent not found)
        for account in &accounts {
            if let Some(parent_id) = account.parent_account_id {
                let parent_exists = accounts.iter().any(|a| a.id == parent_id);
                if !parent_exists {
                    validation_result.is_valid = false;
                    validation_result.errors.push(
                        format!("Account '{}' has non-existent parent", account.account_code)
                    );
                }
            }
        }

        // Check for circular references
        for account in &accounts {
            if self.has_circular_reference(account, &accounts) {
                validation_result.is_valid = false;
                validation_result.errors.push(
                    format!("Circular reference detected for account '{}'", account.account_code)
                );
            }
        }

        // Suggestions for Indonesian accounting
        if !accounts.iter().any(|a| a.account_code.starts_with("211") && a.account_name.contains("PPN")) {
            validation_result.suggestions.push(
                "Consider adding a PPN (VAT) liability account (211xxx)".to_string()
            );
        }

        if !accounts.iter().any(|a| a.account_code.starts_with("212") && a.account_name.contains("PPh")) {
            validation_result.suggestions.push(
                "Consider adding PPh (Income Tax) liability accounts (212xxx)".to_string()
            );
        }

        Ok(validation_result)
    }

    fn validate_account_code(&self, account_code: &str) -> ServiceResult<()> {
        // Indonesian accounting typically uses 3-6 digit codes
        if account_code.len() < 3 || account_code.len() > 6 {
            return Err(ServiceError::Validation(
                "Account code must be between 3-6 digits".to_string()
            ));
        }

        if !account_code.chars().all(|c| c.is_ascii_digit()) {
            return Err(ServiceError::Validation(
                "Account code must contain only digits".to_string()
            ));
        }

        Ok(())
    }

    async fn validate_parent_account(
        &self,
        company_id: Uuid,
        parent_id: Uuid,
        account_type: &AccountType,
    ) -> ServiceResult<()> {
        let parent = sqlx::query!(
            r#"
            SELECT account_type as "account_type: AccountType"
            FROM accounts 
            WHERE id = $1 AND company_id = $2 AND is_active = true
            "#,
            parent_id,
            company_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::Validation("Parent account not found".to_string()))?;

        // Parent must be same account type
        if parent.account_type != *account_type {
            return Err(ServiceError::Validation(
                "Parent account must be the same type".to_string()
            ));
        }

        Ok(())
    }

    fn has_circular_reference(&self, account: &Account, all_accounts: &[Account]) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut current_id = account.parent_account_id;

        while let Some(parent_id) = current_id {
            if visited.contains(&parent_id) || parent_id == account.id {
                return true;
            }
            
            visited.insert(parent_id);
            
            // Find the parent account
            current_id = all_accounts
                .iter()
                .find(|a| a.id == parent_id)
                .and_then(|a| a.parent_account_id);
        }

        false
    }

    fn get_template_accounts(&self, template_name: &str) -> ServiceResult<Vec<TemplateAccount>> {
        match template_name {
            "indonesian_basic" => Ok(self.get_indonesian_basic_template()),
            "indonesian_manufacturing" => Ok(self.get_indonesian_manufacturing_template()),
            "indonesian_trading" => Ok(self.get_indonesian_trading_template()),
            _ => Err(ServiceError::Validation(
                format!("Unknown template: {}", template_name)
            )),
        }
    }

    fn get_indonesian_basic_template(&self) -> Vec<TemplateAccount> {
        vec![
            // Assets
            TemplateAccount {
                account_code: "110".to_string(),
                account_name: "Kas".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "111".to_string(),
                account_name: "Bank".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "120".to_string(),
                account_name: "Piutang Dagang".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "130".to_string(),
                account_name: "Persediaan".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "140".to_string(),
                account_name: "Aset Tetap".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::FixedAsset),
                parent_account_id: None,
            },
            
            // Liabilities
            TemplateAccount {
                account_code: "210".to_string(),
                account_name: "Hutang Dagang".to_string(),
                account_type: AccountType::Liability,
                account_subtype: Some(AccountSubtype::CurrentLiability),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "211".to_string(),
                account_name: "Hutang PPN".to_string(),
                account_type: AccountType::Liability,
                account_subtype: Some(AccountSubtype::CurrentLiability),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "212".to_string(),
                account_name: "Hutang PPh".to_string(),
                account_type: AccountType::Liability,
                account_subtype: Some(AccountSubtype::CurrentLiability),
                parent_account_id: None,
            },
            
            // Equity
            TemplateAccount {
                account_code: "310".to_string(),
                account_name: "Modal".to_string(),
                account_type: AccountType::Equity,
                account_subtype: Some(AccountSubtype::OwnerEquity),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "320".to_string(),
                account_name: "Laba Ditahan".to_string(),
                account_type: AccountType::Equity,
                account_subtype: Some(AccountSubtype::RetainedEarnings),
                parent_account_id: None,
            },
            
            // Revenue
            TemplateAccount {
                account_code: "410".to_string(),
                account_name: "Pendapatan Penjualan".to_string(),
                account_type: AccountType::Revenue,
                account_subtype: Some(AccountSubtype::OperatingRevenue),
                parent_account_id: None,
            },
            
            // Expenses
            TemplateAccount {
                account_code: "510".to_string(),
                account_name: "Harga Pokok Penjualan".to_string(),
                account_type: AccountType::Expense,
                account_subtype: Some(AccountSubtype::CostOfGoodsSold),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "610".to_string(),
                account_name: "Beban Gaji".to_string(),
                account_type: AccountType::Expense,
                account_subtype: Some(AccountSubtype::OperatingExpense),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "620".to_string(),
                account_name: "Beban Sewa".to_string(),
                account_type: AccountType::Expense,
                account_subtype: Some(AccountSubtype::OperatingExpense),
                parent_account_id: None,
            },
        ]
    }

    fn get_indonesian_manufacturing_template(&self) -> Vec<TemplateAccount> {
        let mut accounts = self.get_indonesian_basic_template();
        
        // Add manufacturing-specific accounts
        accounts.extend(vec![
            TemplateAccount {
                account_code: "131".to_string(),
                account_name: "Persediaan Bahan Baku".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "132".to_string(),
                account_name: "Persediaan Barang Dalam Proses".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "133".to_string(),
                account_name: "Persediaan Barang Jadi".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "141".to_string(),
                account_name: "Mesin dan Peralatan".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::FixedAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "630".to_string(),
                account_name: "Beban Overhead Pabrik".to_string(),
                account_type: AccountType::Expense,
                account_subtype: Some(AccountSubtype::OperatingExpense),
                parent_account_id: None,
            },
        ]);
        
        accounts
    }

    fn get_indonesian_trading_template(&self) -> Vec<TemplateAccount> {
        let mut accounts = self.get_indonesian_basic_template();
        
        // Add trading-specific accounts
        accounts.extend(vec![
            TemplateAccount {
                account_code: "121".to_string(),
                account_name: "Piutang Lain-lain".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "122".to_string(),
                account_name: "Uang Muka Pembelian".to_string(),
                account_type: AccountType::Asset,
                account_subtype: Some(AccountSubtype::CurrentAsset),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "213".to_string(),
                account_name: "Uang Muka Penjualan".to_string(),
                account_type: AccountType::Liability,
                account_subtype: Some(AccountSubtype::CurrentLiability),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "420".to_string(),
                account_name: "Diskon Penjualan".to_string(),
                account_type: AccountType::Revenue,
                account_subtype: Some(AccountSubtype::OperatingRevenue),
                parent_account_id: None,
            },
            TemplateAccount {
                account_code: "520".to_string(),
                account_name: "Diskon Pembelian".to_string(),
                account_type: AccountType::Expense,
                account_subtype: Some(AccountSubtype::CostOfGoodsSold),
                parent_account_id: None,
            },
        ]);
        
        accounts
    }
}

#[derive(Debug)]
struct TemplateAccount {
    account_code: String,
    account_name: String,
    account_type: AccountType,
    account_subtype: Option<AccountSubtype>,
    parent_account_id: Option<Uuid>,
}