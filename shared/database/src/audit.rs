use sqlx::{PgPool, Transaction, Postgres};
use uuid::Uuid;
use serde_json::Value;

pub struct AuditLogger {
    pool: PgPool,
}

impl AuditLogger {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_activity(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        table_name: &str,
        record_id: Uuid,
        action: &str,
        old_values: Option<Value>,
        new_values: Option<Value>,
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