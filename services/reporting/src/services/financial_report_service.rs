use crate::models::*;
use common::{ServiceResult, ServiceError};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct FinancialReportService {
    db: PgPool,
}

impl FinancialReportService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn generate_balance_sheet(
        &self,
        company_id: Uuid,
        as_of_date: NaiveDate,
    ) -> ServiceResult<BalanceSheetReport> {
        // Get account balances as of date
        let account_balances = sqlx::query!(
            r#"
            SELECT 
                a.id,
                a.account_code,
                a.account_name,
                a.account_type as "account_type: String",
                a.account_subtype as "account_subtype: Option<String>",
                COALESCE(ab.ending_balance, 0) as balance
            FROM accounts a
            LEFT JOIN account_balances ab ON a.id = ab.account_id 
                AND ab.balance_date <= $1
                AND ab.company_id = $2
            WHERE a.company_id = $2 
                AND a.is_active = true
                AND a.account_type IN ('ASSET', 'LIABILITY', 'EQUITY')
            ORDER BY a.account_code
            "#,
            as_of_date,
            company_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut assets = BalanceSheetSection {
            title: "ASET".to_string(),
            total: Decimal::ZERO,
            subsections: HashMap::new(),
        };

        let mut liabilities = BalanceSheetSection {
            title: "KEWAJIBAN".to_string(),
            total: Decimal::ZERO,
            subsections: HashMap::new(),
        };

        let mut equity = BalanceSheetSection {
            title: "EKUITAS".to_string(),
            total: Decimal::ZERO,
            subsections: HashMap::new(),
        };

        // Process account balances
        for row in account_balances {
            let account_item = AccountItem {
                account_id: row.id,
                account_code: row.account_code,
                account_name: row.account_name,
                balance: row.balance.unwrap_or(Decimal::ZERO),
            };

            match row.account_type.as_str() {
                "ASSET" => {
                    let subtype = row.account_subtype.unwrap_or("OTHER_ASSET".to_string());
                    let subsection = assets.subsections.entry(subtype.clone()).or_insert_with(|| {
                        BalanceSheetSubsection {
                            title: Self::get_indonesian_account_subtype_name(&subtype),
                            accounts: Vec::new(),
                            total: Decimal::ZERO,
                        }
                    });
                    subsection.total += account_item.balance;
                    subsection.accounts.push(account_item);
                    assets.total += subsection.total;
                },
                "LIABILITY" => {
                    let subtype = row.account_subtype.unwrap_or("CURRENT_LIABILITY".to_string());
                    let subsection = liabilities.subsections.entry(subtype.clone()).or_insert_with(|| {
                        BalanceSheetSubsection {
                            title: Self::get_indonesian_account_subtype_name(&subtype),
                            accounts: Vec::new(),
                            total: Decimal::ZERO,
                        }
                    });
                    subsection.total += account_item.balance;
                    subsection.accounts.push(account_item);
                    liabilities.total += subsection.total;
                },
                "EQUITY" => {
                    let subtype = row.account_subtype.unwrap_or("OWNER_EQUITY".to_string());
                    let subsection = equity.subsections.entry(subtype.clone()).or_insert_with(|| {
                        BalanceSheetSubsection {
                            title: Self::get_indonesian_account_subtype_name(&subtype),
                            accounts: Vec::new(),
                            total: Decimal::ZERO,
                        }
                    });
                    subsection.total += account_item.balance;
                    subsection.accounts.push(account_item);
                    equity.total += subsection.total;
                },
                _ => {}
            }
        }

        let total_liabilities_and_equity = liabilities.total + equity.total;

        Ok(BalanceSheetReport {
            company_id,
            as_of_date,
            assets,
            liabilities,
            equity,
            total_assets: assets.total,
            total_liabilities_and_equity,
            is_balanced: (assets.total - total_liabilities_and_equity).abs() < Decimal::new(1, 2),
            generated_at: chrono::Utc::now(),
        })
    }

    pub async fn generate_income_statement(
        &self,
        company_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ServiceResult<IncomeStatementReport> {
        let account_balances = sqlx::query!(
            r#"
            SELECT 
                a.id,
                a.account_code,
                a.account_name,
                a.account_type as "account_type: String",
                a.account_subtype as "account_subtype: Option<String>",
                SUM(
                    CASE 
                        WHEN a.account_type = 'REVENUE' THEN jel.credit_amount - jel.debit_amount
                        WHEN a.account_type = 'EXPENSE' THEN jel.debit_amount - jel.credit_amount
                        ELSE 0
                    END
                ) as net_amount
            FROM accounts a
            LEFT JOIN journal_entry_lines jel ON a.id = jel.account_id
            LEFT JOIN journal_entries je ON jel.journal_entry_id = je.id
            WHERE a.company_id = $1 
                AND a.is_active = true
                AND a.account_type IN ('REVENUE', 'EXPENSE')
                AND je.entry_date >= $2 
                AND je.entry_date <= $3
                AND je.is_posted = true
            GROUP BY a.id, a.account_code, a.account_name, a.account_type, a.account_subtype
            ORDER BY a.account_code
            "#,
            company_id,
            start_date,
            end_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut revenue = IncomeStatementSection {
            title: "PENDAPATAN".to_string(),
            total: Decimal::ZERO,
            subsections: HashMap::new(),
        };

        let mut expenses = IncomeStatementSection {
            title: "BEBAN".to_string(),
            total: Decimal::ZERO,
            subsections: HashMap::new(),
        };

        // Process account balances
        for row in account_balances {
            let net_amount = row.net_amount.unwrap_or(Decimal::ZERO);
            let account_item = AccountItem {
                account_id: row.id,
                account_code: row.account_code,
                account_name: row.account_name,
                balance: net_amount,
            };

            match row.account_type.as_str() {
                "REVENUE" => {
                    let subtype = row.account_subtype.unwrap_or("OPERATING_REVENUE".to_string());
                    let subsection = revenue.subsections.entry(subtype.clone()).or_insert_with(|| {
                        IncomeStatementSubsection {
                            title: Self::get_indonesian_account_subtype_name(&subtype),
                            accounts: Vec::new(),
                            total: Decimal::ZERO,
                        }
                    });
                    subsection.total += account_item.balance;
                    subsection.accounts.push(account_item);
                },
                "EXPENSE" => {
                    let subtype = row.account_subtype.unwrap_or("OPERATING_EXPENSE".to_string());
                    let subsection = expenses.subsections.entry(subtype.clone()).or_insert_with(|| {
                        IncomeStatementSubsection {
                            title: Self::get_indonesian_account_subtype_name(&subtype),
                            accounts: Vec::new(),
                            total: Decimal::ZERO,
                        }
                    });
                    subsection.total += account_item.balance;
                    subsection.accounts.push(account_item);
                },
                _ => {}
            }
        }

        // Calculate totals
        revenue.total = revenue.subsections.values().map(|s| s.total).sum();
        expenses.total = expenses.subsections.values().map(|s| s.total).sum();
        
        let gross_profit = revenue.total;
        let net_income = gross_profit - expenses.total;

        Ok(IncomeStatementReport {
            company_id,
            period_start: start_date,
            period_end: end_date,
            revenue,
            expenses,
            gross_profit,
            net_income,
            generated_at: chrono::Utc::now(),
        })
    }

    pub async fn generate_trial_balance(
        &self,
        company_id: Uuid,
        as_of_date: NaiveDate,
    ) -> ServiceResult<TrialBalanceReport> {
        let account_balances = sqlx::query!(
            r#"
            SELECT 
                a.id,
                a.account_code,
                a.account_name,
                a.account_type as "account_type: String",
                a.normal_balance,
                COALESCE(SUM(jel.debit_amount), 0) as total_debits,
                COALESCE(SUM(jel.credit_amount), 0) as total_credits
            FROM accounts a
            LEFT JOIN journal_entry_lines jel ON a.id = jel.account_id
            LEFT JOIN journal_entries je ON jel.journal_entry_id = je.id
            WHERE a.company_id = $1 
                AND a.is_active = true
                AND (je.entry_date <= $2 OR je.entry_date IS NULL)
                AND (je.is_posted = true OR je.is_posted IS NULL)
            GROUP BY a.id, a.account_code, a.account_name, a.account_type, a.normal_balance
            ORDER BY a.account_code
            "#,
            company_id,
            as_of_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut accounts = Vec::new();
        let mut total_debits = Decimal::ZERO;
        let mut total_credits = Decimal::ZERO;

        for row in account_balances {
            let debits = row.total_debits.unwrap_or(Decimal::ZERO);
            let credits = row.total_credits.unwrap_or(Decimal::ZERO);
            
            let balance = match row.normal_balance.as_str() {
                "DEBIT" => debits - credits,
                "CREDIT" => credits - debits,
                _ => debits - credits,
            };

            let (debit_balance, credit_balance) = if balance >= Decimal::ZERO {
                (balance, Decimal::ZERO)
            } else {
                (Decimal::ZERO, balance.abs())
            };

            accounts.push(TrialBalanceAccount {
                account_id: row.id,
                account_code: row.account_code,
                account_name: row.account_name,
                account_type: row.account_type,
                debit_balance,
                credit_balance,
            });

            total_debits += debit_balance;
            total_credits += credit_balance;
        }

        Ok(TrialBalanceReport {
            company_id,
            as_of_date,
            accounts,
            total_debits,
            total_credits,
            is_balanced: (total_debits - total_credits).abs() < Decimal::new(1, 2),
            generated_at: chrono::Utc::now(),
        })
    }

    fn get_indonesian_account_subtype_name(subtype: &str) -> String {
        match subtype {
            "CURRENT_ASSET" => "Aset Lancar".to_string(),
            "FIXED_ASSET" => "Aset Tetap".to_string(),
            "OTHER_ASSET" => "Aset Lainnya".to_string(),
            "CURRENT_LIABILITY" => "Kewajiban Lancar".to_string(),
            "LONG_TERM_LIABILITY" => "Kewajiban Jangka Panjang".to_string(),
            "OWNER_EQUITY" => "Modal Pemilik".to_string(),
            "RETAINED_EARNINGS" => "Laba Ditahan".to_string(),
            "OPERATING_REVENUE" => "Pendapatan Operasional".to_string(),
            "NON_OPERATING_REVENUE" => "Pendapatan Non-Operasional".to_string(),
            "COST_OF_GOODS_SOLD" => "Harga Pokok Penjualan".to_string(),
            "OPERATING_EXPENSE" => "Beban Operasional".to_string(),
            "NON_OPERATING_EXPENSE" => "Beban Non-Operasional".to_string(),
            _ => subtype.to_string(),
        }
    }
}