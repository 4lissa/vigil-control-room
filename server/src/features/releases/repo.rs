use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::releases::model::{Release, ReleaseState, ReleaseStep};
use crate::shared::error::AppError;

#[derive(sqlx::FromRow)]
struct ReleaseRow {
    id: Uuid,
    team_id: Uuid,
    name: String,
    state: String,
    created_by: Option<Uuid>,
    created_at: OffsetDateTime,
    completed_at: Option<OffsetDateTime>,
}

impl From<ReleaseRow> for Release {
    fn from(row: ReleaseRow) -> Self {
        Self {
            id: row.id,
            team_id: row.team_id,
            name: row.name,
            state: release_state_from_str(&row.state),
            created_by: row.created_by,
            created_at: row.created_at,
            completed_at: row.completed_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ReleaseStepRow {
    id: Uuid,
    release_id: Uuid,
    name: String,
    position: i32,
    validated_at: Option<OffsetDateTime>,
    validated_by: Option<Uuid>,
}

impl From<ReleaseStepRow> for ReleaseStep {
    fn from(row: ReleaseStepRow) -> Self {
        Self {
            id: row.id,
            release_id: row.release_id,
            name: row.name,
            position: row.position,
            validated_at: row.validated_at,
            validated_by: row.validated_by,
        }
    }
}

fn release_state_from_str(s: &str) -> ReleaseState {
    match s {
        "created" => ReleaseState::Created,
        "in_progress" => ReleaseState::InProgress,
        "completed" => ReleaseState::Completed,
        "cancelled" => ReleaseState::Cancelled,
        "blocked" => ReleaseState::Blocked,
        _ => unreachable!("unexpected release_state value from database: {}", s),
    }
}

fn release_state_to_str(state: &ReleaseState) -> &'static str {
    match state {
        ReleaseState::Created => "created",
        ReleaseState::InProgress => "in_progress",
        ReleaseState::Completed => "completed",
        ReleaseState::Cancelled => "cancelled",
        ReleaseState::Blocked => "blocked",
    }
}

pub async fn insert_release(
    executor: impl sqlx::PgExecutor<'_>,
    id: Uuid,
    team_id: Uuid,
    name: &str,
    created_by: Uuid,
) -> Result<Release, AppError> {
    let row = sqlx::query_as!(
        ReleaseRow,
        r#"
        INSERT INTO releases (id, team_id, name, created_by)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id, team_id, name,
            state::TEXT as "state!",
            created_by, created_at, completed_at
        "#,
        id,
        team_id,
        name,
        created_by,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to insert release");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn insert_release_step(
    executor: impl sqlx::PgExecutor<'_>,
    id: Uuid,
    release_id: Uuid,
    name: &str,
    position: i32,
) -> Result<ReleaseStep, AppError> {
    let row = sqlx::query_as!(
        ReleaseStepRow,
        r#"
        INSERT INTO release_steps (id, release_id, name, position)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        id,
        release_id,
        name,
        position,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %release_id, "failed to insert release step");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn find_release_by_id(
    pool: &PgPool,
    release_id: Uuid,
) -> Result<Option<Release>, AppError> {
    let row = sqlx::query_as!(
        ReleaseRow,
        r#"
        SELECT
            id, team_id, name,
            state::TEXT as "state!",
            created_by, created_at, completed_at
        FROM releases
        WHERE id = $1
        "#,
        release_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %release_id, "failed to find release by id");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn find_releases_by_team_id(
    pool: &PgPool,
    team_id: Uuid,
) -> Result<Vec<Release>, AppError> {
    let rows = sqlx::query_as!(
        ReleaseRow,
        r#"
        SELECT
            id, team_id, name,
            state::TEXT as "state!",
            created_by, created_at, completed_at
        FROM releases
        WHERE team_id = $1
        ORDER BY created_at DESC
        "#,
        team_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to find releases by team id");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn find_steps_by_release_id(
    pool: &PgPool,
    release_id: Uuid,
) -> Result<Vec<ReleaseStep>, AppError> {
    let rows = sqlx::query_as!(
        ReleaseStepRow,
        r#"
        SELECT *
        FROM release_steps
        WHERE release_id = $1
        ORDER BY position ASC
        "#,
        release_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %release_id, "failed to find release steps");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn find_step_by_id(
    pool: &PgPool,
    step_id: Uuid,
) -> Result<Option<ReleaseStep>, AppError> {
    let row = sqlx::query_as!(
        ReleaseStepRow,
        r#"SELECT * FROM release_steps WHERE id = $1"#,
        step_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %step_id, "failed to find release step by id");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn update_release_state(
    executor: impl sqlx::PgExecutor<'_>,
    release_id: Uuid,
    new_state: &ReleaseState,
    completed_at: Option<OffsetDateTime>,
) -> Result<Release, AppError> {
    let state_str = release_state_to_str(new_state);
    let row = sqlx::query_as!(
        ReleaseRow,
        r#"
        UPDATE releases
        SET state = $2::release_state, completed_at = $3
        WHERE id = $1
        RETURNING
            id, team_id, name,
            state::TEXT as "state!",
            created_by, created_at, completed_at
        "#,
        release_id,
        state_str as &str,
        completed_at,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %release_id, "failed to update release state");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn update_step_validated(
    executor: impl sqlx::PgExecutor<'_>,
    step_id: Uuid,
    validated_by: Uuid,
) -> Result<ReleaseStep, AppError> {
    let row = sqlx::query_as!(
        ReleaseStepRow,
        r#"
        UPDATE release_steps
        SET validated_at = now(), validated_by = $2
        WHERE id = $1
        RETURNING *
        "#,
        step_id,
        validated_by,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %step_id, "failed to update release step");
        AppError::InternalError
    })?;

    Ok(row.into())
}
