use axum::http::HeaderMap;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;
use crate::models::*;
use super::ServiceRegistry;
use common::{ServiceResult, ServiceError};

pub struct BalanceSheetGenerator<'a> {
    service_registry: &'a ServiceRegistry,
    http_client: &'a reqwest::Client,
}

impl<'a> BalanceSheetGenerator<'a> {
    pub fn new(service_registry: &'a ServiceRegistry, http_client: &'a reqwest::Client) -> Self {
        Self {
            service_registry,
            http_client,
        }
    }

    pub async fn generate(
        &self,
        company_id: Uuid,
        user_id: Uuid,
        period_end: NaiveDate,
        headers: &HeaderMap,
    ) -> ServiceResult<FinancialReport> {
        tracing::info!("Generating balance sheet for company {} as of {}", company_id, period_end);

        let endpoint = format!("/account-balances?as_of_date={}", period_end);
        let response = self.service_registry
            .call_service(self.http_client, "general-ledger", &endpoint, headers)
            .await?;
        
        let account_balances = response.as_array()
            .ok_or_else(|| ServiceError::ExternalService(
                "Invalid response format from general ledger".to_string()
            ))?;

        let mut current_assets = HashMap::new();
        let mut non_current_assets = HashMap::new();
        let mut current_liabilities = HashMap::new();
        let mut non_current_liabilities = HashMap::new();
        let mut equity_accounts = HashMap::new();

        let mut total_assets = Decimal::ZERO;
        let mut total_liabilities = Decimal::ZERO;
        let mut total_equity = Decimal::ZERO;

        for balance in account_balances {
            let account_name = balance.get("account_name")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown Account")
                .to_string();
            
            let account_type = balance.get("account_type")
                .and_then(|t| t.as_str())
                .unwrap_or("");
            
            let account_code = balance.get("account_code")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            
            let balance_amount = balance.get("balance")
                .and_then(|b| b.as_str())
                .and_then(|s| s.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);

            match account_type {
                "ASSET" => {
                    if account_code.starts_with("11") || account_code.starts_with("12") {
                        current_assets.insert(account_name, balance_amount);
                    } else {
                        non_current_assets.insert(account_name, balance_amount);
                    }
                    total_assets += balance_amount;
                }
                "LIABILITY" => {
                    if account_code.starts_with("20") || account_code.starts_with("21") {
                        current_liabilities.insert(account_name, balance_amount);
                    } else {
                        non_current_liabilities.insert(account_name, balance_amount);
                    }
                    total_liabilities += balance_amount;
                }
                "EQUITY" => {
                    equity_accounts.insert(account_name, balance_amount);
                    total_equity += balance_amount;
                }
                _ => {}
            }
        }

        let balance_sheet = BalanceSheetData {
            assets: BalanceSheetSection {
                current: current_assets,
                non_current: non_current_assets,
                total: total_assets,
            },
            liabilities: BalanceSheetSection {
                current: current_liabilities,
                non_current: non_current_liabilities,
                total: total_liabilities,
            },
            equity: BalanceSheetSection {
                current: equity_accounts,
                non_current: HashMap::new(),
                total: total_equity,
            },
            total_assets,
            total_liabilities,
            total_equity,
            is_balanced: (total_assets - (total_liabilities + total_equity)).abs() < Decimal::new(1, 2),
        };

        Ok(FinancialReport {
            company_id,
            report_type: "balance_sheet".to_string(),
            period_start: period_end,
            period_end,
            generated_at: chrono::Utc::now(),
            generated_by: user_id,
            data: serde_json::to_value(balance_sheet).unwrap(),
        })
    }
}

pub struct IncomeStatementGenerator<'a> {
    service_registry: &'a ServiceRegistry,
    http_client: &'a reqwest::Client,
}

impl<'a> IncomeStatementGenerator<'a> {
    pub fn new(service_registry: &'a ServiceRegistry, http_client: &'a reqwest::Client) -> Self {
        Self {
            service_registry,
            http_client,
        }
    }

    pub async fn generate(
        &self,
        company_id: Uuid,
        user_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
        headers: &HeaderMap,
    ) -> ServiceResult<FinancialReport> {
        tracing::info!("Generating income statement for company {} from {} to {}", 
            company_id, period_start, period_end);

        let endpoint = format!("/account-balances?as_of_date={}", period_end);
        let response = self.service_registry
            .call_service(self.http_client, "general-ledger", &endpoint, headers)
            .await?;
        
        let account_balances = response.as_array()
            .ok_or_else(|| ServiceError::ExternalService(
                "Invalid response format from general ledger".to_string()
            ))?;

        let mut revenue = HashMap::new();
        let mut cost_of_goods_sold = HashMap::new();
        let mut operating_expenses = HashMap::new();
        let mut other_income = HashMap::new();
        let mut other_expenses = HashMap::new();
        
        let mut total_revenue = Decimal::ZERO;
        let mut total_cogs = Decimal::ZERO;
        let mut total_operating_expenses = Decimal::ZERO;
        let mut total_other_income = Decimal::ZERO;
        let mut total_other_expenses = Decimal::ZERO;

        for balance in account_balances {
            let account_name = balance.get("account_name")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown Account")
                .to_string();
            
            let account_type = balance.get("account_type")
                .and_then(|t| t.as_str())
                .unwrap_or("");
            
            let account_code = balance.get("account_code")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            
            let balance_amount = balance.get("balance")
                .and_then(|b| b.as_str())
                .and_then(|s| s.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);

            match account_type {
                "REVENUE" => {
                    revenue.insert(account_name, balance_amount);
                    total_revenue += balance_amount;
                }
                "EXPENSE" => {
                    if account_code.starts_with("5") {
                        cost_of_goods_sold.insert(account_name, balance_amount);
                        total_cogs += balance_amount;
                    } else if account_code.starts_with("6") {
                        operating_expenses.insert(account_name, balance_amount);
                        total_operating_expenses += balance_amount;
                    } else if account_code.starts_with("7") {
                        other_expenses.insert(account_name, balance_amount);
                        total_other_expenses += balance_amount;
                    }
                }
                _ => {}
            }
        }

        let gross_profit = total_revenue - total_cogs;
        let operating_income = gross_profit - total_operating_expenses;
        let net_income_before_tax = operating_income + total_other_income - total_other_expenses;
        let tax_expense = Decimal::ZERO; // Would calculate from tax service
        let net_income = net_income_before_tax - tax_expense;

        let income_statement = IncomeStatementData {
            revenue,
            cost_of_goods_sold,
            gross_profit,
            operating_expenses,
            operating_income,
            other_income,
            other_expenses,
            net_income_before_tax,
            tax_expense,
            net_income,
        };

        Ok(FinancialReport {
            company_id,
            report_type: "income_statement".to_string(),
            period_start,
            period_end,
            generated_at: chrono::Utc::now(),
            generated_by: user_id,
            data: serde_json::to_value(income_statement).unwrap(),
        })
    }
}

// Similar implementations for other generators...
pub struct CashFlowGenerator<'a> {
    service_registry: &'a ServiceRegistry,
    http_client: &'a reqwest::Client,
}

pub struct TrialBalanceGenerator<'a> {
    service_registry: &'a ServiceRegistry,
    http_client: &'a reqwest::Client,
}

pub struct FinancialSummaryGenerator<'a> {
    service_registry: &'a ServiceRegistry,
    http_client: &'a reqwest::Client,
}

pub struct ComparativeAnalysisGenerator<'a> {
    service_registry: &'a ServiceRegistry,
    http_client: &'a reqwest::Client,
}

// Implement similar patterns for other generators...