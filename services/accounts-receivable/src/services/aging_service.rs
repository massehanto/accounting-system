use crate::models::*;
use common::{ServiceResult, ServiceError};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct AgingService {
    db: PgPool,
}

impl AgingService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn generate_customer_aging_report(
        &self,
        company_id: Uuid,
        as_of_date: Option<NaiveDate>,
    ) -> ServiceResult<CustomerAgingReport> {
        let report_date = as_of_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        let aging_data = sqlx::query!(
            r#"  
            SELECT 
                c.id as customer_id,
                c.customer_name,
                c.credit_limit,
                ci.id as invoice_id,
                ci.invoice_number,
                ci.invoice_date,
                ci.due_date,
                ci.total_amount - ci.paid_amount as outstanding_amount,
                $2 - ci.due_date as days_overdue
            FROM customer_invoices ci
            JOIN customers c ON ci.customer_id = c.id
            WHERE ci.company_id = $1 
                  AND ci.status != 'PAID'
                  AND ci.status != 'CANCELLED'
                  AND ci.total_amount > ci.paid_amount
            ORDER BY c.customer_name, ci.due_date
            "#,
            company_id,
            report_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut customer_details: HashMap<Uuid, CustomerAgingDetail> = HashMap::new();
        let mut summary = AgingSummary {
            current: Decimal::ZERO,
            days_31_60: Decimal::ZERO,
            days_61_90: Decimal::ZERO,
            over_90_days: Decimal::ZERO,
            total_outstanding: Decimal::ZERO,
            invoice_count: 0,
        };

        for row in aging_data {
            let outstanding = row.outstanding_amount.unwrap_or(Decimal::ZERO);
            let days_overdue = row.days_overdue.unwrap_or(0);
            
            // Update summary
            summary.total_outstanding += outstanding;
            summary.invoice_count += 1;

            match days_overdue {
                d if d <= 30 => summary.current += outstanding,
                d if d <= 60 => summary.days_31_60 += outstanding,
                d if d <= 90 => summary.days_61_90 += outstanding,
                _ => summary.over_90_days += outstanding,
            }

            // Update customer detail
            let customer_detail = customer_details.entry(row.customer_id).or_insert_with(|| {
                CustomerAgingDetail {
                    customer_id: row.customer_id,
                    customer_name: row.customer_name.clone(),
                    credit_limit: row.credit_limit,
                    current: Decimal::ZERO,
                    days_31_60: Decimal::ZERO,
                    days_61_90: Decimal::ZERO,
                    over_90_days: Decimal::ZERO,
                    total_outstanding: Decimal::ZERO,
                    credit_utilization: 0.0,
                    invoices: Vec::new(),
                }
            });

            customer_detail.total_outstanding += outstanding;
            match days_overdue {
                d if d <= 30 => customer_detail.current += outstanding,
                d if d <= 60 => customer_detail.days_31_60 += outstanding,
                d if d <= 90 => customer_detail.days_61_90 += outstanding,
                _ => customer_detail.over_90_days += outstanding,
            }

            customer_detail.invoices.push(InvoiceAgingItem {
                invoice_id: row.invoice_id,
                invoice_number: row.invoice_number,
                invoice_date: row.invoice_date,
                due_date: row.due_date,
                days_overdue,
                outstanding_amount: outstanding,
            });
        }

        // Calculate credit utilization for each customer
        for customer in customer_details.values_mut() {
            if customer.credit_limit > Decimal::ZERO {
                customer.credit_utilization = 
                    (customer.total_outstanding / customer.credit_limit).to_string()
                    .parse().unwrap_or(0.0) * 100.0;
            }
        }

        let report = CustomerAgingReport {
            company_id,
            report_date,
            summary,
            customer_details: customer_details.into_values().collect(),
            generated_at: chrono::Utc::now(),
        };

        tracing::info!("Generated customer aging report for company {} with {} customers", 
            company_id, report.customer_details.len());

        Ok(report)
    }

    pub async fn get_customers_over_credit_limit(
        &self,
        company_id: Uuid,
    ) -> ServiceResult<Vec<CustomerCreditInfo>> {
        let results = sqlx::query!(
            r#"
            SELECT 
                c.id,
                c.customer_name,
                c.credit_limit,
                COALESCE(SUM(ci.total_amount - ci.paid_amount), 0) as current_outstanding
            FROM customers c
            LEFT JOIN customer_invoices ci ON c.id = ci.customer_id 
                AND ci.status != 'CANCELLED' AND ci.total_amount > ci.paid_amount
            WHERE c.company_id = $1 AND c.is_active = true
            GROUP BY c.id, c.customer_name, c.credit_limit
            HAVING c.credit_limit > 0 AND COALESCE(SUM(ci.total_amount - ci.paid_amount), 0) > c.credit_limit
            ORDER BY (COALESCE(SUM(ci.total_amount - ci.paid_amount), 0) - c.credit_limit) DESC
            "#,
            company_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut customers_over_limit = Vec::new();

        for row in results {
            let current_outstanding = row.current_outstanding.unwrap_or(Decimal::ZERO);
            let available_credit = row.credit_limit - current_outstanding;
            let credit_utilization = if row.credit_limit > Decimal::ZERO {
                (current_outstanding / row.credit_limit * Decimal::new(100, 0))
                    .to_string().parse().unwrap_or(0.0)
            } else {
                0.0
            };

            customers_over_limit.push(CustomerCreditInfo {
                customer_id: row.id,
                customer_name: row.customer_name,
                credit_limit: row.credit_limit,
                current_outstanding,
                available_credit,
                credit_utilization,
                outstanding_invoices: 0, // Would need separate query
                days_past_due: None,     // Would need separate query
                credit_status: "OVER_LIMIT".to_string(),
            });
        }

        Ok(customers_over_limit)
    }
}