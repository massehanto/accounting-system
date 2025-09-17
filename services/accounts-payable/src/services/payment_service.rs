use crate::models::*;
use common::{ServiceResult, ServiceError};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PaymentService {
    db: PgPool,
    audit_logger: database::audit::AuditLogger,
}

impl PaymentService {
    pub fn new(db: PgPool) -> Self {
        let audit_logger = database::audit::AuditLogger::new(db.clone());
        Self { db, audit_logger }
    }

    pub async fn process_payment(
        &self,
        invoice_id: Uuid,
        company_id: Uuid,
        payment: PaymentRequest,
        user_id: Uuid,
    ) -> ServiceResult<VendorInvoice> {
        if payment.payment_amount <= Decimal::ZERO {
            return Err(ServiceError::Validation(
                "Payment amount must be positive".to_string()
            ));
        }

        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        // Get current invoice details
        let current_invoice = sqlx::query!(
            r#"
            SELECT vi.total_amount, vi.paid_amount, vi.status as "status_str",
                   v.vendor_name, vi.invoice_number
            FROM vendor_invoices vi
            JOIN vendors v ON vi.vendor_id = v.id
            WHERE vi.id = $1 AND vi.company_id = $2
            "#,
            invoice_id,
            company_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Invoice not found".to_string()))?;

        let remaining_amount = current_invoice.total_amount - current_invoice.paid_amount;
        
        if payment.payment_amount > remaining_amount {
            return Err(ServiceError::Validation(
                format!("Payment amount ({}) exceeds remaining balance ({})", 
                    payment.payment_amount, remaining_amount)
            ));
        }

        let new_paid_amount = current_invoice.paid_amount + payment.payment_amount;
        let new_status = if new_paid_amount >= current_invoice.total_amount {
            InvoiceStatus::Paid
        } else {
            InvoiceStatus::Approved // Partial payment
        };

        // Record the payment
        let payment_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO vendor_payments (
                id, invoice_id, company_id, payment_amount, payment_date, 
                payment_method, bank_account_id, payment_reference, created_by, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            "#,
            payment_id,
            invoice_id,
            company_id,
            payment.payment_amount,
            payment.payment_date,
            payment.payment_method,
            payment.bank_account_id,
            payment.payment_reference,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Update invoice payment status
        let updated_invoice = sqlx::query!(
            r#"
            UPDATE vendor_invoices 
            SET paid_amount = $1,
                status = $2::invoice_status,
                updated_at = NOW()
            WHERE id = $3 AND company_id = $4
            RETURNING id, company_id, vendor_id, invoice_number, invoice_date, due_date,
                      subtotal, tax_amount, total_amount, paid_amount,
                      status as "status_str", description, journal_entry_id, created_at, updated_at
            "#,
            new_paid_amount,
            new_status.to_string(),
            invoice_id,
            company_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Log audit trail
        self.audit_logger.log_activity(
            &mut tx,
            "vendor_invoices",
            invoice_id,
            "PAYMENT",
            Some(serde_json::json!({
                "old_paid_amount": current_invoice.paid_amount,
                "old_status": current_invoice.status_str
            })),
            Some(serde_json::json!({
                "new_paid_amount": new_paid_amount,
                "new_status": new_status.to_string(),
                "payment_amount": payment.payment_amount,
                "payment_method": payment.payment_method
            })),
            user_id,
        ).await.map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        let status = updated_invoice.status_str.as_deref()
            .and_then(|s| s.parse::<InvoiceStatus>().ok())
            .unwrap_or(InvoiceStatus::Draft);

        let invoice = VendorInvoice {
            id: updated_invoice.id,
            company_id: updated_invoice.company_id,
            vendor_id: updated_invoice.vendor_id,
            vendor_name: Some(current_invoice.vendor_name),
            invoice_number: updated_invoice.invoice_number,
            invoice_date: updated_invoice.invoice_date,
            due_date: updated_invoice.due_date,
            subtotal: updated_invoice.subtotal,
            tax_amount: updated_invoice.tax_amount,
            total_amount: updated_invoice.total_amount,
            paid_amount: updated_invoice.paid_amount,
            outstanding_amount: updated_invoice.total_amount - updated_invoice.paid_amount,
            status,
            description: updated_invoice.description,
            journal_entry_id: updated_invoice.journal_entry_id,
            created_at: updated_invoice.created_at,
            updated_at: updated_invoice.updated_at,
        };

        tracing::info!("Processed payment of {} for invoice {} by user {}", 
            payment.payment_amount, current_invoice.invoice_number, user_id);

        Ok(invoice)
    }

    pub async fn get_payment_history(
        &self,
        invoice_id: Uuid,
        company_id: Uuid,
    ) -> ServiceResult<Vec<VendorPayment>> {
        let payments = sqlx::query_as!(
            VendorPayment,
            r#"
            SELECT 
                id, invoice_id, company_id, payment_amount, payment_date,
                payment_method, bank_account_id, payment_reference,
                created_by, created_at
            FROM vendor_payments
            WHERE invoice_id = $1 AND company_id = $2
            ORDER BY payment_date DESC, created_at DESC
            "#,
            invoice_id,
            company_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(ServiceError::Database)?;

        Ok(payments)
    }

    pub async fn reverse_payment(
        &self,
        payment_id: Uuid,
        company_id: Uuid,
        reason: String,
        user_id: Uuid,
    ) -> ServiceResult<()> {
        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        // Get payment details
        let payment = sqlx::query!(
            r#"
            SELECT invoice_id, payment_amount
            FROM vendor_payments
            WHERE id = $1 AND company_id = $2
            "#,
            payment_id,
            company_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Payment not found".to_string()))?;

        // Update invoice paid amount
        sqlx::query!(
            r#"
            UPDATE vendor_invoices 
            SET paid_amount = paid_amount - $1,
                status = CASE 
                    WHEN (paid_amount - $1) <= 0 THEN 'APPROVED'::invoice_status
                    ELSE status
                END,
                updated_at = NOW()
            WHERE id = $2 AND company_id = $3
            "#,
            payment.payment_amount,
            payment.invoice_id,
            company_id
        )
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Mark payment as reversed
        sqlx::query!(
            r#"
            UPDATE vendor_payments 
            SET is_reversed = true, 
                reversal_reason = $1,
                reversed_by = $2,
                reversed_at = NOW()
            WHERE id = $3 AND company_id = $4
            "#,
            reason,
            user_id,
            payment_id,
            company_id
        )
        .execute(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        // Log audit trail
        self.audit_logger.log_activity(
            &mut tx,
            "vendor_payments",
            payment_id,
            "REVERSE",
            None,
            Some(serde_json::json!({
                "reason": reason,
                "payment_amount": payment.payment_amount
            })),
            user_id,
        ).await.map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        tracing::info!("Reversed payment {} for invoice {} by user {}", 
            payment_id, payment.invoice_id, user_id);

        Ok(())
    }
}