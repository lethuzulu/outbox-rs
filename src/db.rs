use chrono::DateTime;
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::{
    errors::OutboxError,
    types::{EventType, MessageStatus, OutboxMessage},
};

#[derive(Debug)]
pub struct Db {
    pub pool: PgPool,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self, OutboxError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), OutboxError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| OutboxError::Database(e.into()))?;
        Ok(())
    }
}

// atomicity helper
impl Db {
    pub async fn with_transaction<'a, F, Fut, T>(&'a self, f: F) -> Result<T, OutboxError>
    where
        F: FnOnce(Transaction<'a, Postgres>) -> Fut,
        Fut: Future<Output = Result<(T, Transaction<'a, Postgres>), OutboxError>>,
    {
        let tx = self.pool.begin().await?;
        let (result, tx) = f(tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}

impl Db {
    pub async fn insert_mesage(
        tx: &mut Transaction<'_, Postgres>,
        event_type: &EventType,
        payload: Value,
        aggregate_id: &str,
    ) -> Result<Uuid, OutboxError> {
        let id = sqlx::query_scalar!(r#"INSERT INTO outbox_messages (event_type, payload, aggregate_id) VALUES ($1, $2, $3) RETURNING id"#, event_type.as_str(),payload,aggregate_id).fetch_one(&mut **tx).await?;
        Ok(id)
    }
}

impl Db {
    pub async fn count_by_status(&self, status: &str) -> Result<i64, OutboxError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM outbox_messages WHERE status = $1",
            status
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        Ok(count)
    }
}

impl Db {
    // claims a batch of pending rows atomically
    pub async fn poll(
        &self,
        lock_secs: i64,
        batch_size: i64,
    ) -> Result<Vec<OutboxMessage>, OutboxError> {
        let rows = sqlx::query!(
                r#"
                UPDATE outbox_messages
                SET status = 'processing', attempts = attempts + 1, locked_until = now() + ($1 || 'seconds')::interval
                WHERE id IN (
                    SELECT id FROM outbox_messages
                    WHERE status = 'pending'
                    AND (locked_until IS NULL OR locked_until < now())
                    ORDER BY created_at
                    LIMIT $2
                    FOR UPDATE SKIP LOCKED
                )
                RETURNING id, event_type, payload, aggregate_id, attempts
                "#,
                lock_secs.to_string(),
                batch_size
            )
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|r| {
                Ok(OutboxMessage {
                    id: r.id,
                    event_type: EventType::new(r.event_type)?,
                    payload: r.payload,
                    aggregate_id: r.aggregate_id,
                    status: MessageStatus::Processing,
                    attempts: r.attempts,
                    published_at: None,
                    created_at: DateTime::default(),
                })
            })
            .collect()
    }
}

impl Db {
    pub async fn mark_published(&self, id: Uuid) -> Result<(), OutboxError> {
        sqlx::query!(
            r#"
            UPDATE outbox_messages
            SET status = 'published', published_at = now(), locked_until = NULL
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), OutboxError> {
        sqlx::query!(
            r#"
            UPDATE outbox_messages
            SET status = 'failed', last_error = $1, locked_until = NULL
            WHERE  id = $2
            "#,
            error,
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
