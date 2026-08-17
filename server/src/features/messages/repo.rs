use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::messages::model::Message;
use crate::shared::error::AppError;

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: String,
    created_at: OffsetDateTime,
}

impl From<MessageRow> for Message {
    fn from(row: MessageRow) -> Self {
        Self {
            id: row.id,
            sender_id: row.sender_id,
            recipient_id: row.recipient_id,
            content: row.content,
            created_at: row.created_at,
        }
    }
}

pub async fn insert_message(
    executor: impl sqlx::PgExecutor<'_>,
    id: Uuid,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: &str,
) -> Result<Message, AppError> {
    let row = sqlx::query_as!(
        MessageRow,
        r#"
        INSERT INTO messages (id, sender_id, recipient_id, content)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        id,
        sender_id,
        recipient_id,
        content,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %sender_id, %recipient_id, "failed to insert message");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn find_conversation(
    pool: &PgPool,
    user_a: Uuid,
    user_b: Uuid,
) -> Result<Vec<Message>, AppError> {
    let rows = sqlx::query_as!(
        MessageRow,
        r#"
        SELECT *
        FROM messages
        WHERE (sender_id = $1 AND recipient_id = $2)
           OR (sender_id = $2 AND recipient_id = $1)
        ORDER BY created_at ASC
        "#,
        user_a,
        user_b,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %user_a, %user_b, "failed to find conversation");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn find_latest_message_per_conversation(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Message>, AppError> {
    let rows = sqlx::query_as!(
        MessageRow,
        r#"
        SELECT id, sender_id, recipient_id, content, created_at
        FROM (
            SELECT DISTINCT ON (counterpart)
                id, sender_id, recipient_id, content, created_at,
                CASE WHEN sender_id = $1 THEN recipient_id ELSE sender_id END AS counterpart
            FROM messages
            WHERE sender_id = $1 OR recipient_id = $1
            ORDER BY counterpart, created_at DESC
        ) AS latest_per_conversation
        ORDER BY created_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %user_id, "failed to find conversations for user");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}
