use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::rule_engine::model::{ReactionType, Rule};
use crate::shared::error::AppError;

#[derive(sqlx::FromRow)]
struct RuleRow {
    id: Uuid,
    team_id: Uuid,
    name: String,
    enabled: bool,
    trigger_service: String,
    trigger_event: String,
    trigger_filters: serde_json::Value,
    encrypted_webhook_secret: Vec<u8>,
    reaction_type: String,
    reaction_payload: serde_json::Value,
    created_by: Option<Uuid>,
    created_at: OffsetDateTime,
}

impl TryFrom<RuleRow> for Rule {
    type Error = AppError;

    fn try_from(row: RuleRow) -> Result<Self, AppError> {
        let reaction_type = ReactionType::parse(&row.reaction_type).ok_or_else(|| {
            tracing::error!(reaction_type = %row.reaction_type, "unknown reaction_type value from database");
            AppError::InternalError
        })?;

        Ok(Self {
            id: row.id,
            team_id: row.team_id,
            name: row.name,
            enabled: row.enabled,
            trigger_service: row.trigger_service,
            trigger_event: row.trigger_event,
            trigger_filters: row.trigger_filters,
            encrypted_webhook_secret: row.encrypted_webhook_secret,
            reaction_type,
            reaction_payload: row.reaction_payload,
            created_by: row.created_by,
            created_at: row.created_at,
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_rule(
    pool: &PgPool,
    id: Uuid,
    team_id: Uuid,
    name: &str,
    enabled: bool,
    trigger_service: &str,
    trigger_event: &str,
    trigger_filters: &serde_json::Value,
    encrypted_webhook_secret: &[u8],
    reaction_type: ReactionType,
    reaction_payload: &serde_json::Value,
    created_by: Uuid,
) -> Result<Rule, AppError> {
    let reaction_type_str = reaction_type.as_str();

    let row = sqlx::query_as!(
        RuleRow,
        r#"
        INSERT INTO rules (
            id, team_id, name, enabled, trigger_service, trigger_event, trigger_filters,
            encrypted_webhook_secret, reaction_type, reaction_payload, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
        id,
        team_id,
        name,
        enabled,
        trigger_service,
        trigger_event,
        trigger_filters,
        encrypted_webhook_secret,
        reaction_type_str,
        reaction_payload,
        created_by,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to insert rule");
        AppError::InternalError
    })?;

    row.try_into()
}

pub async fn find_rule_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Rule>, AppError> {
    let row = sqlx::query_as!(RuleRow, r#"SELECT * FROM rules WHERE id = $1"#, id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, %id, "failed to find rule by id");
            AppError::InternalError
        })?;

    row.map(TryInto::try_into).transpose()
}

pub async fn find_rules_by_team_id(pool: &PgPool, team_id: Uuid) -> Result<Vec<Rule>, AppError> {
    let rows = sqlx::query_as!(
        RuleRow,
        r#"SELECT * FROM rules WHERE team_id = $1 ORDER BY created_at DESC"#,
        team_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to find rules by team id");
        AppError::InternalError
    })?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn update_rule_enabled(pool: &PgPool, id: Uuid, enabled: bool) -> Result<Rule, AppError> {
    let row = sqlx::query_as!(
        RuleRow,
        r#"UPDATE rules SET enabled = $2 WHERE id = $1 RETURNING *"#,
        id,
        enabled,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %id, "failed to update rule enabled state");
        AppError::InternalError
    })?;

    row.try_into()
}

pub async fn delete_rule(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query!(r#"DELETE FROM rules WHERE id = $1"#, id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, %id, "failed to delete rule");
            AppError::InternalError
        })?;

    Ok(())
}
