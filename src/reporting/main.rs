// src/reporting/main.rs
use axum::{
    extract::{Query, State},
    http::{StatusCode, HeaderMap},
    response::{Json, IntoResponse},
    routing::get,
    Router,
};
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
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

#[derive(Debug, Serialize, Deserialize)]
struct FinancialReport {
    company_id: Uuid,
    report_type: String,
    period_start: NaiveDate,
    period_end: NaiveDate,
    generated_at: chrono::DateTime<chrono::Utc>,
    generated_by: Uuid,
    data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct BalanceSheetData {
    assets: BalanceSheetSection,
    liabilities: BalanceSheetSection,
    equity: BalanceSheetSection,
    total_assets: Decimal,
    total_liabilities: Decimal,
    total_equity: Decimal,
    is_balanced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct BalanceSheetSection {
    current: HashMap<String, Decimal>,
    non_current: HashMap<String, Decimal>,
    total: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
struct IncomeStatementData {
    revenue: HashMap<String, Decimal>,
    cost_of_goods_sold: HashMap<String, Decimal>,
    gross_profit: Decimal,
    operating_expenses: HashMap<String, Decimal>,
    operating_income: Decimal,
    other_income: HashMap<String, Decimal>,
    other_expenses: HashMap<String, Decimal>,
    net_income_before_tax: Decimal,
    tax_expense: Decimal,
    net_income: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
struct CashFlowData {
    operating_activities: HashMap<String, Decimal>,
    investing_activities: HashMap<String, Decimal>,
    financing_activities: HashMap<String, Decimal>,
    net_cash_from_operations: Decimal,
    net_cash_from_investing: Decimal,
    net_cash_from_financing: Decimal,
    net_change_in_cash: Decimal,
    beginning_cash: Decimal,
    ending_cash: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrialBalanceData {
    accounts: Vec<TrialBalanceAccount>,
    total_debits: Decimal,
    total_credits: Decimal,
    is_balanced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrialBalanceAccount {
    account_code: String,
    account_name: String,
    account_type: String,
    debit_balance: Decimal,
    credit_balance: Decimal,
}

#[derive(Clone)]
struct AppState {
    http_client: Client,
    services: HashMap<String, String>,
}

impl AppState {
    async fn call_service(
        &self,
        service: &str,
        endpoint: &str,
        headers: &HeaderMap,
    ) -> Result<serde_json::Value, StatusCode> {
        let service_url = self.services.get(service)
            .ok_or_else(|| {
                error!("Service {} not configured", service);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        let url = format!("{}{}", service_url, endpoint);
        
        let mut request = self.http_client.get(&url).timeout(Duration::from_secs(30));
        
        // Forward authentication headers
        if let Some(user_id) = headers.get("X-User-ID") {
            request = request.header("X-User-ID", user_id);
        }
        if let Some(company_id) = headers.get("X-Company-ID") {
            request = request.header("X-Company-ID", company_id);
        }
        if let Some(auth) = headers.get("Authorization") {
            request = request.header("Authorization", auth);
        }
        
        let response = request.send().await
            .map_err(|e| {
                error!("Failed to call {} service at {}: {}", service, url, e);
                StatusCode::BAD_GATEWAY
            })?;
        
        if !response.status().is_success() {
            error!("Service {} returned error status: {}", service, response.status());
            return Err(StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));
        }
        
        response.json().await
            .map_err(|e| {
                error!("Failed to parse response from {} service: {}", service, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }

    async fn get_account_balances(
        &self,
        headers: &HeaderMap,
        as_of_date: &str,
    ) -> Result<Vec<serde_json::Value>, StatusCode> {
        let endpoint = format!("/account-balances?as_of_date={}", as_of_date);
        let response = self.call_service("general-ledger", &endpoint, headers).await?;
        
        response.as_array()
            .ok_or_else(|| {
                error!("Expected array response from account balances");
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(|arr| arr.clone())
    }

    async fn get_trial_balance(
        &self,
        headers: &HeaderMap,
        as_of_date: &str,
    ) -> Result<serde_json::Value, StatusCode> {
        let endpoint = format!("/trial-balance?as_of_date={}", as_of_date);
        self.call_service("general-ledger", &endpoint, headers).await
    }

    async fn get_chart_of_accounts(
        &self,
        headers: &HeaderMap,
    ) -> Result<Vec<serde_json::Value>, StatusCode> {
        let response = self.call_service("chart-of-accounts", "/accounts", headers).await?;
        
        response.as_array()
            .ok_or_else(|| {
                error!("Expected array response from chart of accounts");
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(|arr| arr.clone())
    }
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "reporting-service",
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn generate_balance_sheet(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FinancialReport>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_end = params.get("period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    info!("Generating balance sheet for company {} as of {}", company_id, period_end);

    // Get account balances
    let account_balances = state.get_account_balances(&headers, &period_end.to_string()).await?;
    
    // Initialize balance sheet sections
    let mut current_assets = HashMap::new();
    let mut non_current_assets = HashMap::new();
    let mut current_liabilities = HashMap::new();
    let mut non_current_liabilities = HashMap::new();
    let mut equity_accounts = HashMap::new();

    let mut total_assets = Decimal::ZERO;
    let mut total_liabilities = Decimal::ZERO;
    let mut total_equity = Decimal::ZERO;

    // Categorize accounts and calculate balances
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
                // Simple classification - in practice, you'd have more sophisticated logic
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
        is_balanced: (total_assets - (total_liabilities + total_equity)).abs() < Decimal::new(1, 2), // Allow 1 cent difference
    };

    let report = FinancialReport {
        company_id,
        report_type: "balance_sheet".to_string(),
        period_start: period_end,
        period_end,
        generated_at: chrono::Utc::now(),
        generated_by: user_id,
        data: serde_json::to_value(balance_sheet).unwrap(),
    };

    info!("Balance sheet generated successfully for company {}", company_id);
    Ok(Json(report))
}

async fn generate_income_statement(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FinancialReport>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_start = params.get("period_start")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let period_end = params.get("period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if period_start >= period_end {
        return Err(StatusCode::BAD_REQUEST);
    }

    info!("Generating income statement for company {} from {} to {}", 
        company_id, period_start, period_end);

    // Get account balances for the period
    let account_balances = state.get_account_balances(&headers, &period_end.to_string()).await?;
    
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

    // Categorize accounts
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
    let tax_expense = Decimal::ZERO; // Would calculate based on tax transactions
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

    let report = FinancialReport {
        company_id,
        report_type: "income_statement".to_string(),
        period_start,
        period_end,
        generated_at: chrono::Utc::now(),
        generated_by: user_id,
        data: serde_json::to_value(income_statement).unwrap(),
    };

    info!("Income statement generated successfully for company {}", company_id);
    Ok(Json(report))
}

async fn generate_cash_flow_statement(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FinancialReport>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_start = params.get("period_start")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let period_end = params.get("period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Generating cash flow statement for company {} from {} to {}", 
        company_id, period_start, period_end);

    // TODO: Implement actual cash flow calculation from journal entries
    // This is a simplified version - would need to analyze cash account movements
    
    let mut operating_activities = HashMap::new();
    let mut investing_activities = HashMap::new();
    let mut financing_activities = HashMap::new();

    // Sample data - in practice, would calculate from actual transactions
    operating_activities.insert("Net Income".to_string(), Decimal::ZERO);
    operating_activities.insert("Depreciation".to_string(), Decimal::ZERO);
    operating_activities.insert("Changes in Working Capital".to_string(), Decimal::ZERO);
    
    investing_activities.insert("Equipment Purchases".to_string(), Decimal::ZERO);
    investing_activities.insert("Equipment Sales".to_string(), Decimal::ZERO);
    
    financing_activities.insert("Loan Proceeds".to_string(), Decimal::ZERO);
    financing_activities.insert("Loan Payments".to_string(), Decimal::ZERO);
    financing_activities.insert("Owner Contributions".to_string(), Decimal::ZERO);

    let net_cash_from_operations: Decimal = operating_activities.values().sum();
    let net_cash_from_investing: Decimal = investing_activities.values().sum();
    let net_cash_from_financing: Decimal = financing_activities.values().sum();
    
    let net_change_in_cash = net_cash_from_operations + net_cash_from_investing + net_cash_from_financing;
    let beginning_cash = Decimal::ZERO; // Would get from beginning balance
    let ending_cash = beginning_cash + net_change_in_cash;

    let cash_flow = CashFlowData {
        operating_activities,
        investing_activities,
        financing_activities,
        net_cash_from_operations,
        net_cash_from_investing,
        net_cash_from_financing,
        net_change_in_cash,
        beginning_cash,
        ending_cash,
    };

    let report = FinancialReport {
        company_id,
        report_type: "cash_flow_statement".to_string(),
        period_start,
        period_end,
        generated_at: chrono::Utc::now(),
        generated_by: user_id,
        data: serde_json::to_value(cash_flow).unwrap(),
    };

    info!("Cash flow statement generated successfully for company {}", company_id);
    Ok(Json(report))
}

async fn generate_trial_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FinancialReport>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_end = params.get("period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    info!("Generating trial balance for company {} as of {}", company_id, period_end);

    let trial_balance_response = state.get_trial_balance(&headers, &period_end.to_string()).await?;
    
    let accounts = trial_balance_response.get("accounts")
        .and_then(|a| a.as_array())
        .ok_or_else(|| {
            error!("Invalid trial balance response format");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .iter()
        .map(|account| {
            TrialBalanceAccount {
                account_code: account.get("account_code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string(),
                account_name: account.get("account_name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                account_type: account.get("account_type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
                debit_balance: account.get("debit_balance")
                    .and_then(|b| b.as_str())
                    .and_then(|s| s.parse::<Decimal>().ok())
                    .unwrap_or(Decimal::ZERO),
                credit_balance: account.get("credit_balance")
                    .and_then(|b| b.as_str())
                    .and_then(|s| s.parse::<Decimal>().ok())
                    .unwrap_or(Decimal::ZERO),
            }
        })
        .collect();

    let total_debits = trial_balance_response.get("total_debits")
        .and_then(|t| t.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);
    
    let total_credits = trial_balance_response.get("total_credits")
        .and_then(|t| t.as_str())
        .and_then(|s| s.parse::<Decimal>().ok())
        .unwrap_or(Decimal::ZERO);

    let trial_balance = TrialBalanceData {
        accounts,
        total_debits,
        total_credits,
        is_balanced: (total_debits - total_credits).abs() < Decimal::new(1, 2), // Allow 1 cent difference
    };

    let report = FinancialReport {
        company_id,
        report_type: "trial_balance".to_string(),
        period_start: period_end,
        period_end,
        generated_at: chrono::Utc::now(),
        generated_by: user_id,
        data: serde_json::to_value(trial_balance).unwrap(),
    };

    info!("Trial balance generated successfully for company {}", company_id);
    Ok(Json(report))
}

async fn generate_financial_summary(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FinancialReport>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let period_start = params.get("period_start")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let period_end = params.get("period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Generating financial summary for company {} from {} to {}", 
        company_id, period_start, period_end);

    // Get data from multiple services
    let account_balances = state.get_account_balances(&headers, &period_end.to_string()).await?;
    
    let mut total_assets = Decimal::ZERO;
    let mut total_liabilities = Decimal::ZERO;
    let mut total_equity = Decimal::ZERO;
    let mut total_revenue = Decimal::ZERO;
    let mut total_expenses = Decimal::ZERO;

    // Calculate key metrics
    for balance in account_balances {
        let account_type = balance.get("account_type")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        
        let balance_amount = balance.get("balance")
            .and_then(|b| b.as_str())
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or(Decimal::ZERO);

        match account_type {
            "ASSET" => total_assets += balance_amount,
            "LIABILITY" => total_liabilities += balance_amount,
            "EQUITY" => total_equity += balance_amount,
            "REVENUE" => total_revenue += balance_amount,
            "EXPENSE" => total_expenses += balance_amount,
            _ => {}
        }
    }

    let net_income = total_revenue - total_expenses;
    let debt_to_equity_ratio = if total_equity > Decimal::ZERO {
        total_liabilities / total_equity
    } else {
        Decimal::ZERO
    };
    
    let return_on_assets = if total_assets > Decimal::ZERO {
        net_income / total_assets * Decimal::new(100, 0) // Convert to percentage
    } else {
        Decimal::ZERO
    };

    let current_ratio = Decimal::new(100, 0); // Would calculate from current assets/liabilities

    // Get additional data from other services
    let inventory_data = match state.call_service("inventory", "/stock-report", &headers).await {
        Ok(data) => Some(data),
        Err(_) => {
            warn!("Could not fetch inventory data for financial summary");
            None
        }
    };

    let ap_data = match state.call_service("accounts-payable", "/aging-report", &headers).await {
        Ok(data) => Some(data),
        Err(_) => {
            warn!("Could not fetch accounts payable data for financial summary");
            None
        }
    };

    let summary_data = serde_json::json!({
        "period": {
            "start_date": period_start,
            "end_date": period_end,
            "days": (period_end - period_start).num_days()
        },
        "financial_position": {
            "total_assets": total_assets,
            "total_liabilities": total_liabilities,
            "total_equity": total_equity,
            "working_capital": Decimal::ZERO, // Would calculate current assets - current liabilities
        },
        "performance": {
            "total_revenue": total_revenue,
            "total_expenses": total_expenses,
            "net_income": net_income,
            "gross_margin_percentage": if total_revenue > Decimal::ZERO {
                (total_revenue - total_expenses) / total_revenue * Decimal::new(100, 0)
            } else {
                Decimal::ZERO
            }
        },
        "financial_ratios": {
            "current_ratio": current_ratio,
            "debt_to_equity_ratio": debt_to_equity_ratio,
            "return_on_assets_percentage": return_on_assets,
            "asset_turnover_ratio": if total_assets > Decimal::ZERO {
                total_revenue / total_assets
            } else {
                Decimal::ZERO
            }
        },
        "inventory_summary": inventory_data.and_then(|data| data.get("summary").cloned()),
        "accounts_payable_summary": ap_data.and_then(|data| data.get("summary").cloned()),
        "key_insights": [
            {
                "metric": "Profitability",
                "status": if net_income > Decimal::ZERO { "Positive" } else { "Negative" },
                "value": net_income,
                "recommendation": if net_income <= Decimal::ZERO {
                    "Review expenses and revenue strategies"
                } else {
                    "Continue current profitable operations"
                }
            },
            {
                "metric": "Liquidity",
                "status": if current_ratio >= Decimal::new(150, 0) { "Good" } else { "Concerning" },
                "value": current_ratio,
                "recommendation": if current_ratio < Decimal::new(150, 0) {
                    "Improve cash flow and reduce short-term debt"
                } else {
                    "Maintain healthy liquidity levels"
                }
            },
            {
                "metric": "Leverage",
                "status": if debt_to_equity_ratio <= Decimal::new(100, 0) { "Conservative" } else { "High" },
                "value": debt_to_equity_ratio,
                "recommendation": if debt_to_equity_ratio > Decimal::new(100, 0) {
                    "Consider reducing debt levels"
                } else {
                    "Debt levels are manageable"
                }
            }
        ]
    });

    let report = FinancialReport {
        company_id,
        report_type: "financial_summary".to_string(),
        period_start,
        period_end,
        generated_at: chrono::Utc::now(),
        generated_by: user_id,
        data: summary_data,
    };

    info!("Financial summary generated successfully for company {}", company_id);
    Ok(Json(report))
}

async fn generate_comparative_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FinancialReport>, StatusCode> {
    let user_id = extract_user_id(&headers)?;
    let company_id = extract_company_id(&headers)?;
    
    let current_period_start = params.get("current_period_start")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let current_period_end = params.get("current_period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let prior_period_start = params.get("prior_period_start")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let prior_period_end = params.get("prior_period_end")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!("Generating comparative report for company {} comparing {} to {} vs {} to {}", 
        company_id, current_period_start, current_period_end, prior_period_start, prior_period_end);

    // Get balances for both periods
    let current_balances = state.get_account_balances(&headers, &current_period_end.to_string()).await?;
    let prior_balances = state.get_account_balances(&headers, &prior_period_end.to_string()).await?;

    // Create maps for easier comparison
    let mut current_map = HashMap::new();
    let mut prior_map = HashMap::new();

    for balance in current_balances {
        if let (Some(code), Some(amount_str)) = (
            balance.get("account_code").and_then(|c| c.as_str()),
            balance.get("balance").and_then(|b| b.as_str())
        ) {
            if let Ok(amount) = amount_str.parse::<Decimal>() {
                current_map.insert(code.to_string(), amount);
            }
        }
    }

    for balance in prior_balances {
        if let (Some(code), Some(amount_str)) = (
            balance.get("account_code").and_then(|c| c.as_str()),
            balance.get("balance").and_then(|b| b.as_str())
        ) {
            if let Ok(amount) = amount_str.parse::<Decimal>() {
                prior_map.insert(code.to_string(), amount);
            }
        }
    }

    // Calculate variances
    let mut account_comparisons = Vec::new();
    let mut all_accounts = std::collections::HashSet::new();
    
    for key in current_map.keys() {
        all_accounts.insert(key.clone());
    }
    for key in prior_map.keys() {
        all_accounts.insert(key.clone());
    }

    for account_code in all_accounts {
        let current_amount = current_map.get(&account_code).copied().unwrap_or(Decimal::ZERO);
        let prior_amount = prior_map.get(&account_code).copied().unwrap_or(Decimal::ZERO);
        let variance = current_amount - prior_amount;
        let variance_percentage = if prior_amount != Decimal::ZERO {
            (variance / prior_amount) * Decimal::new(100, 0)
        } else if current_amount != Decimal::ZERO {
            Decimal::new(100, 0) // 100% increase from zero
        } else {
            Decimal::ZERO
        };

        account_comparisons.push(serde_json::json!({
            "account_code": account_code,
            "current_period": current_amount,
            "prior_period": prior_amount,
            "variance": variance,
            "variance_percentage": variance_percentage
        }));
    }

    // Sort by absolute variance (largest changes first)
    account_comparisons.sort_by(|a, b| {
        let variance_a = a["variance"].as_str()
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or(Decimal::ZERO)
            .abs();
        let variance_b = b["variance"].as_str()
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or(Decimal::ZERO)
            .abs();
        variance_b.cmp(&variance_a)
    });

    let comparative_data = serde_json::json!({
        "current_period": {
            "start_date": current_period_start,
            "end_date": current_period_end
        },
        "prior_period": {
            "start_date": prior_period_start,
            "end_date": prior_period_end
        },
        "account_comparisons": account_comparisons,
        "summary": {
            "accounts_increased": account_comparisons.iter()
                .filter(|acc| acc["variance"].as_str()
                    .and_then(|s| s.parse::<Decimal>().ok())
                    .unwrap_or(Decimal::ZERO) > Decimal::ZERO)
                .count(),
            "accounts_decreased": account_comparisons.iter()
                .filter(|acc| acc["variance"].as_str()
                    .and_then(|s| s.parse::<Decimal>().ok())
                    .unwrap_or(Decimal::ZERO) < Decimal::ZERO)
                .count(),
            "significant_changes": account_comparisons.iter()
                .filter(|acc| {
                    let variance_pct = acc["variance_percentage"].as_str()
                        .and_then(|s| s.parse::<Decimal>().ok())
                        .unwrap_or(Decimal::ZERO)
                        .abs();
                    variance_pct > Decimal::new(10, 0) // Greater than 10% change
                })
                .take(10)
                .collect::<Vec<_>>()
        }
    });

    let report = FinancialReport {
        company_id,
        report_type: "comparative_analysis".to_string(),
        period_start: current_period_start,
        period_end: current_period_end,
        generated_at: chrono::Utc::now(),
        generated_by: user_id,
        data: comparative_data,
    };

    info!("Comparative report generated successfully for company {}", company_id);
    Ok(Json(report))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,reporting_service=debug")
        .init();

    info!("Starting Reporting Service...");

    // Initialize service URLs from environment variables
    let mut services = HashMap::new();
    
    // Use environment variables with fallbacks
    services.insert(
        "chart-of-accounts".to_string(), 
        std::env::var("CHART_OF_ACCOUNTS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3003".to_string())
    );
    services.insert(
        "general-ledger".to_string(), 
        std::env::var("GENERAL_LEDGER_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3004".to_string())
    );
    services.insert(
        "accounts-payable".to_string(), 
        std::env::var("ACCOUNTS_PAYABLE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3006".to_string())
    );
    services.insert(
        "accounts-receivable".to_string(), 
        std::env::var("ACCOUNTS_RECEIVABLE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3007".to_string())
    );
    services.insert(
        "inventory".to_string(), 
        std::env::var("INVENTORY_MANAGEMENT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3008".to_string())
    );
    services.insert(
        "tax".to_string(), 
        std::env::var("INDONESIAN_TAX_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3005".to_string())
    );

    info!("Configured service endpoints:");
    for (service, url) in &services {
        info!("  {}: {}", service, url);
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    let app_state = Arc::new(AppState {
        http_client: client,
        services,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/balance-sheet", get(generate_balance_sheet))
        .route("/income-statement", get(generate_income_statement))
        .route("/cash-flow", get(generate_cash_flow_statement))
        .route("/trial-balance", get(generate_trial_balance))
        .route("/financial-summary", get(generate_financial_summary))
        .route("/comparative-analysis", get(generate_comparative_report))
        .with_state(app_state);

    let bind_addr = std::env::var("REPORTING_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3009".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Reporting service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}