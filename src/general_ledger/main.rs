// src/general_ledger/main.rs
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::{Json, IntoResponse},
    routing::{get, post, put},
    Router,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Transaction, Postgres, Type, postgres::PgPoolOptions};
use std::{sync::Arc, str::FromStr, env};
use uuid::Uuid;
use validator::Validate;
use tracing::{error, info, warn, debug};

// Database connection function specific to general ledger service
async fn create_database_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("GENERAL_LEDGER_DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("GENERAL_LEDGER_DATABASE_URL must be set"))?;

    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "20".to_string())
        .parse::<u32>()
        .unwrap_or(20);

    let min_connections = env::var("DB_MIN_CONNECTIONS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u32>()
        .unwrap_or(5);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .connect(&database_url)
        .await?;

    info!("Connected to general ledger database: {}", database_url.split('@').last().unwrap_or("unknown"));
    Ok(pool)
}

async fn check_database_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Debit and credit amounts don't balance: debits={debits}, credits={credits}")]
    DebitCreditMismatch { debits: Decimal, credits: Decimal },
    
    #[error("Journal entry cannot be empty")]
    EmptyEntry,
    
    #[error("Invalid line amount - must have either debit or credit, not both")]
    InvalidLineAmount,
    
    #[error("Journal entry is already posted")]
    AlreadyPosted,
    
    #[error("Account not found: {account_id}")]
    AccountNotFound { account_id: Uuid },
    
    #[error("User not authenticated")]
    UserNotAuthenticated,
    
    #[error("Journal entry not found")]
    JournalEntryNotFound,
    
    #[error("Invalid journal entry status")]
    InvalidStatus,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone)]
#[sqlx(type_name = "journal_entry_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JournalEntryStatus {
    Draft,
    PendingApproval,
    Approved,
    Posted,
    Cancelled,
}

impl Default for JournalEntryStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl FromStr for JournalEntryStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DRAFT" => Ok(JournalEntryStatus::Draft),
            "PENDING_APPROVAL" => Ok(JournalEntryStatus::PendingApproval),
            "APPROVED" => Ok(JournalEntryStatus::Approved),
            "POSTED" => Ok(JournalEntryStatus::Posted),
            "CANCELLED" => Ok(JournalEntryStatus::Cancelled),
            _ => Err(format!("Invalid journal entry status: {}", s))
        }
    }
}

impl std::fmt::Display for JournalEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JournalEntryStatus::Draft => write!(f, "DRAFT"),
            JournalEntryStatus::PendingApproval => write!(f, "PENDING_APPROVAL"),
            JournalEntryStatus::Approved => write!(f, "APPROVED"),
            JournalEntryStatus::Posted => write!(f, "POSTED"),
            JournalEntryStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JournalEntry {
    id: Uuid,
    company_id: Uuid,
    entry_number: String,
    entry_date: NaiveDate,
    description: Option<String>,
    reference: Option<String>,
    total_debit: Decimal,
    total_credit: Decimal,
    status: JournalEntryStatus,
    is_posted: bool,
    created_by: Uuid,
    approved_by: Option<Uuid>,
    posted_by: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    approved_at: Option<chrono::DateTime<chrono::Utc>>,
    posted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JournalEntryLine {
    id: Uuid,
    journal_entry_id: Uuid,
    account_id: Uuid,
    account_code: Option<String>,
    account_name: Option<String>,
    description: Option<String>,
    debit_amount: Decimal,
    credit_amount: Decimal,
    line_number: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct CreateJournalEntryRequest {
    company_id: Uuid,
    entry_date: NaiveDate,
    description: Option<String>,
    reference: Option<String>,
    #[validate(length(min = 1, message = "At least one line is required"))]
    lines: Vec<CreateJournalEntryLineRequest>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct CreateJournalEntryLineRequest {
    account_id: Uuid,
    description: Option<String>,
    #[validate(range(min = 0, message = "Debit amount cannot be negative"))]
    debit_amount: Decimal,
    #[validate(range(min = 0, message = "Credit amount cannot be negative"))]
    credit_amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateJournalEntryRequest {
    entry_date: Option<NaiveDate>,
    description: Option<String>,
    reference: Option<String>,
    lines: Option<Vec<CreateJournalEntryLineRequest>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JournalEntryWithLines {
    journal_entry: JournalEntry,
    lines: Vec<JournalEntryLine>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrialBalance {
    company_id: Uuid,
    as_of_date: NaiveDate,
    accounts: Vec<TrialBalanceAccount>,
    total_debits: Decimal,
    total_credits: Decimal,
    is_balanced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrialBalanceAccount {
    account_id: Uuid,
    account_code: String,
    account_name: String,
    account_type: String,
    debit_balance: Decimal,
    credit_balance: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountBalance {
    account_id: Uuid,
    account_code: String,
    account_name: String,
    account_type: String,
    balance: Decimal,
    is_debit_balance: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuditLog {
    id: Uuid,
    table_name: String,
    record_id: Uuid,
    action: String,
    old_values: Option<serde_json::Value>,
    new_values: Option<serde_json::Value>,
    user_id: Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, LedgerError> {
    headers
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(LedgerError::UserNotAuthenticated)
}

fn extract_company_id(headers: &HeaderMap) -> Result<Uuid, LedgerError> {
    headers
        .get("X-Company-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(LedgerError::UserNotAuthenticated)
}

impl AppState {
    async fn log_audit_trail(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        table_name: &str,
        record_id: Uuid,
        action: &str,
        old_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (id, table_name, record_id, action, old_values, new_values, user_id, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            "#,
            Uuid::new_v4(),
            table_name,
            record_id,
            action,
            old_values,
            new_values,
            user_id
        )
        .execute(&mut **tx)
        .await?;
        
        Ok(())
    }
}

fn validate_journal_entry(lines: &[CreateJournalEntryLineRequest]) -> Result<(), LedgerError> {
    if lines.is_empty() {
        return Err(LedgerError::EmptyEntry);
    }

    let total_debits: Decimal = lines.iter().map(|l| l.debit_amount).sum();
    let total_credits: Decimal = lines.iter().map(|l| l.credit_amount).sum();
    
    if total_debits != total_credits {
        return Err(LedgerError::DebitCreditMismatch {
            debits: total_debits,
            credits: total_credits,
        });
    }

    for line in lines {
        let has_debit = line.debit_amount > Decimal::ZERO;
        let has_credit = line.credit_amount > Decimal::ZERO;
        
        if (has_debit && has_credit) || (!has_debit && !has_credit) {
            return Err(LedgerError::InvalidLineAmount);
        }
    }
    
    Ok(())
}

async fn validate_accounts_exist(
    pool: &PgPool,
    account_ids: &[Uuid],
    company_id: Uuid,
) -> Result<(), LedgerError> {
    for &account_id in account_ids {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND company_id = $2 AND is_active = true)",
            account_id,
            company_id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| LedgerError::AccountNotFound { account_id })?
        .unwrap_or(false);

        if !exists {
            return Err(LedgerError::AccountNotFound { account_id });
        }
    }
    
    Ok(())
}

async fn generate_entry_number(
    pool: &PgPool,
    company_id: Uuid,
    entry_date: NaiveDate,
) -> Result<String, sqlx::Error> {
    let entry_number = sqlx::query_scalar!(
        "SELECT generate_entry_number($1, $2)",
        company_id,
        entry_date
    )
    .fetch_one(pool)
    .await?;
    
    Ok(entry_number.unwrap_or_else(|| {
        format!("JE-{}-000001", entry_date.format("%Y"))
    }))
}

async fn create_journal_entry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(mut payload): Json<CreateJournalEntryRequest>,
) -> Result<Json<JournalEntryWithLines>, StatusCode> {
    let user_id = extract_user_id(&headers).map_err(|e| {
        warn!("Failed to extract user ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let header_company_id = extract_company_id(&headers).map_err(|e| {
        warn!("Failed to extract company ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    payload.company_id = header_company_id;

    if let Err(e) = payload.validate() {
        warn!("Journal entry validation failed: {:?}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Err(e) = validate_journal_entry(&payload.lines) {
        warn!("Journal entry business rule validation failed: {}", e);
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let account_ids: Vec<Uuid> = payload.lines.iter().map(|l| l.account_id).collect();
    if let Err(e) = validate_accounts_exist(&state.db, &account_ids, payload.company_id).await {
        warn!("Account validation failed: {}", e);
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let total_debits: Decimal = payload.lines.iter().map(|l| l.debit_amount).sum();
    let total_credits: Decimal = payload.lines.iter().map(|l| l.credit_amount).sum();
    
    let entry_id = Uuid::new_v4();
    
    let entry_number = generate_entry_number(&state.db, payload.company_id, payload.entry_date)
        .await
        .map_err(|e| {
            error!("Failed to generate entry number: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
        payload.company_id,
        entry_number,
        payload.entry_date,
        payload.description,
        payload.reference,
        total_debits,
        total_credits,
        JournalEntryStatus::Draft as JournalEntryStatus,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to create journal entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut lines = Vec::new();
    for (index, line_request) in payload.lines.into_iter().enumerate() {
        let line_id = Uuid::new_v4();
        
        let account_info = sqlx::query!(
            "SELECT account_code, account_name FROM accounts WHERE id = $1",
            line_request.account_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to get account info: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
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
        .map_err(|e| {
            error!("Failed to create journal entry line: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        let line_with_account = JournalEntryLine {
            account_code: account_info.as_ref().map(|a| a.account_code.clone()),
            account_name: account_info.as_ref().map(|a| a.account_name.clone()),
            ..line
        };
        
        lines.push(line_with_account);
    }

    state.log_audit_trail(
        &mut tx,
        "journal_entries",
        entry_id,
        "CREATE",
        None,
        Some(serde_json::to_value(&journal_entry).unwrap()),
        user_id,
    ).await.map_err(|e| {
        error!("Failed to log audit trail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created journal entry: {} for company {} by user {}", 
        entry_number, payload.company_id, user_id);

    Ok(Json(JournalEntryWithLines {
        journal_entry,
        lines,
    }))
}

async fn get_journal_entries(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<JournalEntry>>, StatusCode> {
    let company_id = extract_company_id(&headers).map_err(|e| {
        warn!("Failed to extract company ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let limit: i64 = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50)
        .min(1000);

    let offset: i64 = params.get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    let status_filter = params.get("status");

    let entries = if let Some(status) = status_filter {
        sqlx::query_as!(
            JournalEntry,
            r#"
            SELECT id, company_id, entry_number, entry_date, description, reference,
                   total_debit, total_credit, 
                   status as "status: JournalEntryStatus", is_posted, 
                   created_by, approved_by, posted_by, created_at, approved_at, posted_at
            FROM journal_entries 
            WHERE company_id = $1 AND status = $2::journal_entry_status
            ORDER BY entry_date DESC, entry_number DESC
            LIMIT $3 OFFSET $4
            "#,
            company_id,
            status,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as!(
            JournalEntry,
            r#"
            SELECT id, company_id, entry_number, entry_date, description, reference,
                   total_debit, total_credit, 
                   status as "status: JournalEntryStatus", is_posted, 
                   created_by, approved_by, posted_by, created_at, approved_at, posted_at
            FROM journal_entries 
            WHERE company_id = $1 
            ORDER BY entry_date DESC, entry_number DESC
            LIMIT $2 OFFSET $3
            "#,
            company_id,
            limit,
            offset
        )
        .fetch_all(&state.db)
        .await
    };

    let entries = entries.map_err(|e| {
        error!("Failed to fetch journal entries: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(entries))
}

async fn get_journal_entry_with_lines(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<JournalEntryWithLines>, StatusCode> {
    let company_id = extract_company_id(&headers).map_err(|e| {
        warn!("Failed to extract company ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let journal_entry = sqlx::query_as!(
        JournalEntry,
        r#"
        SELECT id, company_id, entry_number, entry_date, description, reference,
               total_debit, total_credit, 
               status as "status: JournalEntryStatus", is_posted, 
               created_by, approved_by, posted_by, created_at, approved_at, posted_at
        FROM journal_entries 
        WHERE id = $1 AND company_id = $2
        "#,
        id,
        company_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch journal entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let lines = sqlx::query!(
        r#"
        SELECT jel.id, jel.journal_entry_id, jel.account_id, jel.description, 
               jel.debit_amount, jel.credit_amount, jel.line_number,
               a.account_code, a.account_name
        FROM journal_entry_lines jel
        LEFT JOIN accounts a ON jel.account_id = a.id
        WHERE jel.journal_entry_id = $1 
        ORDER BY jel.line_number
        "#,
        id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch journal entry lines: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|row| JournalEntryLine {
        id: row.id,
        journal_entry_id: row.journal_entry_id,
        account_id: row.account_id,
        account_code: row.account_code,
        account_name: row.account_name,
        description: row.description,
        debit_amount: row.debit_amount,
        credit_amount: row.credit_amount,
        line_number: row.line_number,
    })
    .collect();

    Ok(Json(JournalEntryWithLines {
        journal_entry,
        lines,
    }))
}

async fn update_journal_entry_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<JournalEntry>, StatusCode> {
    let user_id = extract_user_id(&headers).map_err(|e| {
        warn!("Failed to extract user ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let company_id = extract_company_id(&headers).map_err(|e| {
        warn!("Failed to extract company ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let new_status_str = params.get("status")
        .ok_or(StatusCode::BAD_REQUEST)?;

    let target_status: JournalEntryStatus = new_status_str.parse()
        .map_err(|e: String| {
            warn!("Invalid status provided: {} - {}", new_status_str, e);
            StatusCode::BAD_REQUEST
        })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let current_entry = sqlx::query!(
        r#"
        SELECT status as "status: JournalEntryStatus", is_posted 
        FROM journal_entries 
        WHERE id = $1 AND company_id = $2
        "#,
        id,
        company_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to check journal entry status: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let can_transition = match (&current_entry.status, &target_status) {
        (JournalEntryStatus::Draft, JournalEntryStatus::PendingApproval) => true,
        (JournalEntryStatus::PendingApproval, JournalEntryStatus::Approved) => true,
        (JournalEntryStatus::PendingApproval, JournalEntryStatus::Draft) => true,
        (JournalEntryStatus::Approved, JournalEntryStatus::Posted) => true,
        (_, JournalEntryStatus::Cancelled) => !current_entry.is_posted,
        _ => false,
    };

    if !can_transition {
        warn!("Invalid status transition from {:?} to {:?}", current_entry.status, target_status);
        return Err(StatusCode::CONFLICT);
    }

    let journal_entry = match target_status {
        JournalEntryStatus::PendingApproval => {
            sqlx::query_as!(
                JournalEntry,
                r#"
                UPDATE journal_entries 
                SET status = $1, approved_by = NULL, approved_at = NULL
                WHERE id = $2 AND company_id = $3
                RETURNING id, company_id, entry_number, entry_date, description, reference,
                          total_debit, total_credit, 
                          status as "status: JournalEntryStatus", is_posted, 
                          created_by, approved_by, posted_by, created_at, approved_at, posted_at
                "#,
                target_status as JournalEntryStatus,
                id,
                company_id
            )
            .fetch_one(&mut *tx)
            .await
        }
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
                target_status as JournalEntryStatus,
                user_id,
                id,
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
                target_status as JournalEntryStatus,
                user_id,
                id,
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
                target_status as JournalEntryStatus,
                id,
                company_id
            )
            .fetch_one(&mut *tx)
            .await
        }
    };

    let journal_entry = journal_entry.map_err(|e| {
        error!("Failed to update journal entry status: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state.log_audit_trail(
        &mut tx,
        "journal_entries",
        id,
        "STATUS_UPDATE",
        Some(serde_json::json!({"old_status": current_entry.status})),
        Some(serde_json::json!({"new_status": target_status})),
        user_id,
    ).await.map_err(|e| {
        error!("Failed to log audit trail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Updated journal entry {} status to {} by user {}", 
        journal_entry.entry_number, target_status, user_id);

    Ok(Json(journal_entry))
}

async fn delete_journal_entry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user_id = extract_user_id(&headers).map_err(|e| {
        warn!("Failed to extract user ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let company_id = extract_company_id(&headers).map_err(|e| {
        warn!("Failed to extract company ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let entry = sqlx::query!(
        r#"
        SELECT entry_number, status as "status: JournalEntryStatus", is_posted 
        FROM journal_entries 
        WHERE id = $1 AND company_id = $2
        "#,
        id,
        company_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to check journal entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    if entry.is_posted || !matches!(entry.status, JournalEntryStatus::Draft) {
        return Err(StatusCode::CONFLICT);
    }

    sqlx::query!(
        "DELETE FROM journal_entry_lines WHERE journal_entry_id = $1",
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete journal entry lines: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "DELETE FROM journal_entries WHERE id = $1 AND company_id = $2",
        id,
        company_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to delete journal entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state.log_audit_trail(
        &mut tx,
        "journal_entries",
        id,
        "DELETE",
        Some(serde_json::json!({"entry_number": entry.entry_number})),
        None,
        user_id,
    ).await.map_err(|e| {
        error!("Failed to log audit trail: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Deleted journal entry {} by user {}", entry.entry_number, user_id);

    Ok(StatusCode::NO_CONTENT)
}

async fn get_trial_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<TrialBalance>, StatusCode> {
    let company_id = extract_company_id(&headers).map_err(|e| {
        warn!("Failed to extract company ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;
    
    let as_of_date = params.get("as_of_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().naive_local().date());

    let accounts_with_balances = sqlx::query!(
        r#"
        SELECT 
            a.id as account_id,
            a.account_code,
            a.account_name,
            a.account_type::text,
            a.normal_balance,
            COALESCE(SUM(
                CASE WHEN jel.debit_amount > 0 THEN jel.debit_amount ELSE 0 END
            ), 0) as total_debits,
            COALESCE(SUM(
                CASE WHEN jel.credit_amount > 0 THEN jel.credit_amount ELSE 0 END
            ), 0) as total_credits
        FROM accounts a
        LEFT JOIN journal_entry_lines jel ON a.id = jel.account_id
        LEFT JOIN journal_entries je ON jel.journal_entry_id = je.id
        WHERE a.company_id = $1 
              AND a.is_active = true
              AND (je.is_posted = true OR je.id IS NULL)
              AND (je.entry_date <= $2 OR je.id IS NULL)
        GROUP BY a.id, a.account_code, a.account_name, a.account_type, a.normal_balance
        ORDER BY a.account_code
        "#,
        company_id,
        as_of_date
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to fetch trial balance data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut accounts = Vec::new();
    let mut total_debits = Decimal::ZERO;
    let mut total_credits = Decimal::ZERO;

    for row in accounts_with_balances {
        let account_total_debits = row.total_debits.unwrap_or(Decimal::ZERO);
        let account_total_credits = row.total_credits.unwrap_or(Decimal::ZERO);
        let normal_balance = row.normal_balance.unwrap_or_else(|| "DEBIT".to_string());
        
        let net_balance = account_total_debits - account_total_credits;
        let (debit_balance, credit_balance) = if normal_balance == "DEBIT" {
            if net_balance >= Decimal::ZERO {
                (net_balance, Decimal::ZERO)
            } else {
                (Decimal::ZERO, -net_balance)
            }
        } else {
            if net_balance <= Decimal::ZERO {
                (Decimal::ZERO, -net_balance)
            } else {
                (net_balance, Decimal::ZERO)
            }
        };

        if debit_balance > Decimal::ZERO || credit_balance > Decimal::ZERO {
            accounts.push(TrialBalanceAccount {
                account_id: row.account_id,
                account_code: row.account_code,
                account_name: row.account_name,
                account_type: row.account_type.unwrap_or_default(),
                debit_balance,
                credit_balance,
            });

            total_debits += debit_balance;
            total_credits += credit_balance;
        }
    }

    let trial_balance = TrialBalance {
        company_id,
        as_of_date,
        accounts,
        total_debits,
        total_credits,
        is_balanced: total_debits == total_credits,
    };

    Ok(Json(trial_balance))
}

async fn get_account_balances(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AccountBalance>>, StatusCode> {
    let company_id = extract_company_id(&headers).map_err(|e| {
        warn!("Failed to extract company ID: {}", e);
        StatusCode::UNAUTHORIZED
    })?;
    
    let as_of_date = params.get("as_of_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().naive_local().date());

    let account_id_filter = params.get("account_id")
        .and_then(|id| Uuid::parse_str(id).ok());

    let balances = if let Some(account_id) = account_id_filter {
        sqlx::query!(
            r#"
            SELECT 
                a.id as account_id,
                a.account_code,
                a.account_name,
                a.account_type::text,
                a.normal_balance,
                COALESCE(SUM(jel.debit_amount), 0) as total_debits,
                COALESCE(SUM(jel.credit_amount), 0) as total_credits
            FROM accounts a
            LEFT JOIN journal_entry_lines jel ON a.id = jel.account_id
            LEFT JOIN journal_entries je ON jel.journal_entry_id = je.id
            WHERE a.id = $1 AND a.company_id = $2 
                  AND a.is_active = true
                  AND (je.is_posted = true OR je.id IS NULL)
                  AND (je.entry_date <= $3 OR je.id IS NULL)
            GROUP BY a.id, a.account_code, a.account_name, a.account_type, a.normal_balance
            "#,
            account_id,
            company_id,
            as_of_date
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query!(
            r#"
            SELECT 
                a.id as account_id,
                a.account_code,
                a.account_name,
                a.account_type::text,
                a.normal_balance,
                COALESCE(SUM(jel.debit_amount), 0) as total_debits,
                COALESCE(SUM(jel.credit_amount), 0) as total_credits
            FROM accounts a
            LEFT JOIN journal_entry_lines jel ON a.id = jel.account_id
            LEFT JOIN journal_entries je ON jel.journal_entry_id = je.id
            WHERE a.company_id = $1 
                  AND a.is_active = true
                  AND (je.is_posted = true OR je.id IS NULL)
                  AND (je.entry_date <= $2 OR je.id IS NULL)
            GROUP BY a.id, a.account_code, a.account_name, a.account_type, a.normal_balance
            ORDER BY a.account_code
            "#,
            company_id,
            as_of_date
        )
        .fetch_all(&state.db)
        .await
    };

    let balances = balances.map_err(|e| {
        error!("Failed to fetch account balances: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let account_balances: Vec<AccountBalance> = balances
        .into_iter()
        .map(|row| {
            let total_debits = row.total_debits.unwrap_or(Decimal::ZERO);
            let total_credits = row.total_credits.unwrap_or(Decimal::ZERO);
            let normal_balance = row.normal_balance.unwrap_or_else(|| "DEBIT".to_string());
            
            let net_balance = total_debits - total_credits;
            let (balance, is_debit_balance) = if normal_balance == "DEBIT" {
                (net_balance, true)
            } else {
                (-net_balance, false)
            };

            AccountBalance {
                account_id: row.account_id,
                account_code: row.account_code,
                account_name: row.account_name,
                account_type: row.account_type.unwrap_or_default(),
                balance: balance.abs(),
                is_debit_balance: if balance >= Decimal::ZERO { is_debit_balance } else { !is_debit_balance },
            }
        })
        .collect();

    Ok(Json(account_balances))
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_healthy = check_database_health(&state.db).await;
    
    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy { 
        StatusCode::OK 
    } else { 
        StatusCode::SERVICE_UNAVAILABLE 
    };
    
    (status_code, Json(serde_json::json!({
        "service": "general-ledger-service",
        "status": status,
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter("info,general_ledger=debug")
        .init();

    info!("Starting General Ledger Service...");

    let pool = create_database_pool().await?;
    let app_state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/journal-entries", post(create_journal_entry))
        .route("/journal-entries", get(get_journal_entries))
        .route("/journal-entries/:id", get(get_journal_entry_with_lines))
        .route("/journal-entries/:id", axum::routing::delete(delete_journal_entry))
        .route("/journal-entries/:id/status", put(update_journal_entry_status))
        .route("/trial-balance", get(get_trial_balance))
        .route("/account-balances", get(get_account_balances))
        .with_state(app_state);

    let bind_addr = std::env::var("GENERAL_LEDGER_SERVICE_BIND")
        .unwrap_or_else(|_| "0.0.0.0:3004".to_string());
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("General Ledger service listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;
    Ok(())
}