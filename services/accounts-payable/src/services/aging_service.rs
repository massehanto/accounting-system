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

    pub async fn generate_aging_report(
        &self,
        company_id: Uuid,
        as_of_date: Option<NaiveDate>,
    ) -> ServiceResult<AgingReport> {
        let report_date = as_of_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        let aging_data = sqlx::query!(
            r#"
            SELECT 
                v.id as vendor_id,
                v.vendor_name,
                vi.id as invoice_id,
                vi.invoice_number,
                vi.invoice_date,
                vi.due_date,
                vi.total_amount - vi.paid_amount as outstanding_amount,
                $2 - vi.due_date as days_overdue
            FROM vendor_invoices vi
            JOIN vendors v ON vi.vendor_id = v.id
            WHERE vi.company_id = $1 
                  AND vi.status != 'PAID'
                  AND vi.status != 'CANCELLED'
                  AND vi.total_amount > vi.paid_amount
            ORDER BY v.vendor_name, vi.due_date
            "#,
            company_id,
            report_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let mut vendor_details: HashMap<Uuid, VendorAgingDetail> = HashMap::new();
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

            // Update vendor detail
            let vendor_detail = vendor_details.entry(row.vendor_id).or_insert_with(|| {
                VendorAgingDetail {
                    vendor_id: row.vendor_id,
                    vendor_name: row.vendor_name.clone(),
                    current: Decimal::ZERO,
                    days_31_60: Decimal::ZERO,
                    days_61_90: Decimal::ZERO,
                    over_90_days: Decimal::ZERO,
                    total_outstanding: Decimal::ZERO,
                    invoices: Vec::new(),
                }
            });

            vendor_detail.total_outstanding += outstanding;
            match days_overdue {
                d if d <= 30 => vendor_detail.current += outstanding,
                d if d <= 60 => vendor_detail.days_31_60 += outstanding,
                d if d <= 90 => vendor_detail.days_61_90 += outstanding,
                _ => vendor_detail.over_90_days += outstanding,
            }

            vendor_detail.invoices.push(InvoiceAgingItem {
                invoice_id: row.invoice_id,
                invoice_number: row.invoice_number,
                invoice_date: row.invoice_date,
                due_date: row.due_date,
                days_overdue,
                outstanding_amount: outstanding,
            });
        }

        let report = AgingReport {
            company_id,
            report_date,
            summary,
            vendor_details: vendor_details.into_values().collect(),
            generated_at: chrono::Utc::now(),
        };

        tracing::info!("Generated aging report for company {} with {} vendors", 
            company_id, report.vendor_details.len());

        Ok(report)
    }

    pub async fn get_overdue_invoices(
        &self,
        company_id: Uuid,
        days_overdue: Option<i32>,
    ) -> ServiceResult<Vec<VendorInvoice>> {
        let cutoff_date = chrono::Utc::now().date_naive() - 
            chrono::Duration::days(days_overdue.unwrap_or(0) as i64);

        let invoices_data = sqlx::query!(
            r#"
            SELECT vi.id, vi.company_id, vi.vendor_id, v.vendor_name, vi.invoice_number, 
                   vi.invoice_date, vi.due_date, vi.subtotal, vi.tax_amount, vi.total_amount, 
                   vi.paid_amount, vi.status as "status_str", vi.description, 
                   vi.journal_entry_id, vi.created_at, vi.updated_at
            FROM vendor_invoices vi
            LEFT JOIN vendors v ON vi.vendor_id = v.id
            WHERE vi.company_id = $1 
                  AND vi.due_date < $2
                  AND vi.status != 'PAID'
                  AND vi.status != 'CANCELLED'
                  AND vi.total_amount > vi.paid_amount
            ORDER BY vi.due_date ASC
            "#,
            company_id,
            cutoff_date
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        let invoices: Vec<VendorInvoice> = invoices_data
            .into_iter()
            .map(|row| {
                let status = row.status_str.as_deref()
                    .and_then(|s| s.parse::<InvoiceStatus>().ok())
                    .unwrap_or(InvoiceStatus::Draft);

                VendorInvoice {
                    id: row.id,
                    company_id: row.company_id,
                    vendor_id: row.vendor_id,
                    vendor_name: row.vendor_name,
                    invoice_number: row.invoice_number,
                    invoice_date: row.invoice_date,
                    due_date: row.due_date,
                    subtotal: row.subtotal,
                    tax_amount: row.tax_amount,
                    total_amount: row.total_amount,
                    paid_amount: row.paid_amount,
                    outstanding_amount: row.total_amount - row.paid_amount,
                    status,
                    description: row.description,
                    journal_entry_id: row.journal_entry_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }
            })
            .collect();

        Ok(invoices)
    }
}