use crate::models::*;
use common::{ServiceResult, ServiceError};
use sqlx::PgPool;
use uuid::Uuid;

pub struct JournalService {
    db: PgPool,
}

impl JournalService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_entry(
        &self,
        request: CreateJournalEntryRequest,
        user_id: Uuid,
    ) -> ServiceResult<JournalEntryWithLines> {
        // Validate business rules
        super::validation::validate_journal_entry(&request.lines)?;
        
        let account_ids: Vec<Uuid> = request.lines.iter().map(|l| l.account_id).collect();
        super::validation::validate_accounts_exist(&self.db, &account_ids, request.company_id).await?;

        let mut tx = self.db.begin().await
            .map_err(|e| ServiceError::Database(e))?;
        
        let total_debits: rust_decimal::Decimal = request.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: rust_decimal::Decimal = request.lines.iter().map(|l| l.credit_amount).sum();
        
        let entry_id = Uuid::new_v4();
        
        let entry_number = self.generate_entry_number(request.company_id, request.entry_date).await?;

        let journal_entry = sqlx::query_as!(
            JournalEntry,
            r#"
            INSERT INTO journal_entries 
            (id, company_id, entry_number, entry_date, description, reference, total_debit, total_credit, 
             status, is_posted, created_by, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, $10, NOW())
            RETURNING id, company_id, entry_number, entry_date, description, reference, 
                      total_debit, total_credit, 
                      status as "status: JournalEntryStatus", is_posted, 
                      created_by, approved_by, posted_by, created_at, approved_at, posted_at
            "#,
            entry_id,
            request.company_id,
            entry_number,
            request.entry_date,
            request.description,
            request.reference,
            total_debits,
            total_credits,
            JournalEntryStatus::Draft as JournalEntryStatus,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(ServiceError::Database)?;

        let mut lines = Vec::new();
        for (index, line_request) in request.lines.into_iter().enumerate() {
            let line_id = Uuid::new_v4();
            
            let account_info = sqlx::query!(
                "SELECT account_code, account_name FROM accounts WHERE id = $1",
                line_request.account_id
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(ServiceError::Database)?;
            
            let line = sqlx::query_as!(
                JournalEntryLine,
                r#"
                INSERT INTO journal_entry_lines 
                (id, journal_entry_id, account_id, description, debit_amount, credit_amount, line_number)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, journal_entry_id, account_id, description, debit_amount, credit_amount, line_number
                "#,
                line_id,
                entry_id,
                line_request.account_id,
                line_request.description,
                line_request.debit_amount,
                line_request.credit_amount,
                (index + 1) as i32
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(ServiceError::Database)?;
            
            let line_with_account = JournalEntryLine {
                account_code: account_info.as_ref().map(|a| a.account_code.clone()),
                account_name: account_info.as_ref().map(|a| a.account_name.clone()),
                ..line
            };
            
            lines.push(line_with_account);
        }

        tx.commit().await.map_err(ServiceError::Database)?;

        Ok(JournalEntryWithLines {
            journal_entry,
            lines,
        })
    }

    async fn generate_entry_number(
        &self,
        company_id: Uuid,
        entry_date: chrono::NaiveDate,
    ) -> ServiceResult<String> {
        let entry_number = sqlx::query_scalar!(
            "SELECT generate_entry_number($1, $2)",
            company_id,
            entry_date
        )
        .fetch_one(&self.db)
        .await
        .map_err(ServiceError::Database)?;
        
        Ok(entry_number.unwrap_or_else(|| {
            format!("JE-{}-000001", entry_date.format("%Y"))
        }))
    }

    pub async fn update_status(
        &self,
        entry_id: Uuid,
        company_id: Uuid,
        new_status: JournalEntryStatus,
        user_id: Uuid,
    ) -> ServiceResult<JournalEntry> {
        let mut tx = self.db.begin().await.map_err(ServiceError::Database)?;

        let current_entry = sqlx::query!(
            r#"
            SELECT status as "status: JournalEntryStatus", is_posted 
            FROM journal_entries 
            WHERE id = $1 AND company_id = $2
            "#,
            entry_id,
            company_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(ServiceError::Database)?
        .ok_or_else(|| ServiceError::NotFound("Journal entry not found".to_string()))?;

        // Validate status transition
        if !self.can_transition_status(&current_entry.status, &new_status, current_entry.is_posted) {
            return Err(ServiceError::Validation(
                format!("Invalid status transition from {:?} to {:?}", current_entry.status, new_status)
            ));
        }

        let journal_entry = match new_status {
            JournalEntryStatus::Approved => {
                sqlx::query_as!(
                    JournalEntry,
                    r#"
                    UPDATE journal_entries 
                    SET status = $1, approved_by = $2, approved_at = NOW()
                    WHERE id = $3 AND company_id = $4
                    RETURNING id, company_id, entry_number, entry_date, description, reference,
                              total_debit, total_credit, 
                              status as "status: JournalEntryStatus", is_posted, 
                              created_by, approved_by, posted_by, created_at, approved_at, posted_at
                    "#,
                    new_status as JournalEntryStatus,
                    user_id,
                    entry_id,
                    company_id
                )
                .fetch_one(&mut *tx)
                .await
            }
            JournalEntryStatus::Posted => {
                sqlx::query_as!(
                    JournalEntry,
                    r#"
                    UPDATE journal_entries 
                    SET status = $1, is_posted = true, posted_by = $2, posted_at = NOW()
                    WHERE id = $3 AND company_id = $4
                    RETURNING id, company_id, entry_number, entry_date, description, reference,
                              total_debit, total_credit, 
                              status as "status: JournalEntryStatus", is_posted, 
                              created_by, approved_by, posted_by, created_at, approved_at, posted_at
                    "#,
                    new_status as JournalEntryStatus,
                    user_id,
                    entry_id,
                    company_id
                )
                .fetch_one(&mut *tx)
                .await
            }
            _ => {
                sqlx::query_as!(
                    JournalEntry,
                    r#"
                    UPDATE journal_entries 
                    SET status = $1
                    WHERE id = $2 AND company_id = $3
                    RETURNING id, company_id, entry_number, entry_date, description, reference,
                              total_debit, total_credit, 
                              status as "status: JournalEntryStatus", is_posted, 
                              created_by, approved_by, posted_by, created_at, approved_at, posted_at
                    "#,
                    new_status as JournalEntryStatus,
                    entry_id,
                    company_id
                )
                .fetch_one(&mut *tx)
                .await
            }
        };

        let journal_entry = journal_entry.map_err(ServiceError::Database)?;

        tx.commit().await.map_err(ServiceError::Database)?;

        Ok(journal_entry)
    }

    fn can_transition_status(
        &self,
        current: &JournalEntryStatus,
        target: &JournalEntryStatus,
        is_posted: bool,
    ) -> bool {
        match (current, target) {
            (JournalEntryStatus::Draft, JournalEntryStatus::PendingApproval) => true,
            (JournalEntryStatus::PendingApproval, JournalEntryStatus::Approved) => true,
            (JournalEntryStatus::PendingApproval, JournalEntryStatus::Draft) => true,
            (JournalEntryStatus::Approved, JournalEntryStatus::Posted) => true,
            (_, JournalEntryStatus::Cancelled) => !is_posted,
            _ => false,
        }
    }
}