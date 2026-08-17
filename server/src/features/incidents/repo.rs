use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::incidents::model::{
    Incident, IncidentState, Reaction, ReactionEmoji, ReactionWithUsername, Severity, TimelineEntry,
};
use crate::shared::error::AppError;

#[derive(sqlx::FromRow)]
struct IncidentRow {
    id: Uuid,
    team_id: Uuid,
    title: String,
    description: String,
    state: String,
    severity: String,
    created_by: Option<Uuid>,
    assigned_to: Option<Uuid>,
    release_id: Option<Uuid>,
    created_at: OffsetDateTime,
    resolved_at: Option<OffsetDateTime>,
}

impl From<IncidentRow> for Incident {
    fn from(row: IncidentRow) -> Self {
        Self {
            id: row.id,
            team_id: row.team_id,
            title: row.title,
            description: row.description,
            state: incident_state_from_str(&row.state),
            severity: severity_from_str(&row.severity),
            created_by: row.created_by,
            assigned_to: row.assigned_to,
            release_id: row.release_id,
            created_at: row.created_at,
            resolved_at: row.resolved_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TimelineEntryRow {
    id: Uuid,
    incident_id: Uuid,
    author_id: Option<Uuid>,
    content: String,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
}

impl From<TimelineEntryRow> for TimelineEntry {
    fn from(row: TimelineEntryRow) -> Self {
        Self {
            id: row.id,
            incident_id: row.incident_id,
            author_id: row.author_id,
            content: row.content,
            created_at: row.created_at,
            edited_at: row.edited_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ReactionRow {
    id: Uuid,
    entry_id: Uuid,
    user_id: Uuid,
    emoji: String,
    created_at: OffsetDateTime,
}

impl From<ReactionRow> for Reaction {
    fn from(row: ReactionRow) -> Self {
        Self {
            id: row.id,
            entry_id: row.entry_id,
            user_id: row.user_id,
            emoji: reaction_emoji_from_str(&row.emoji),
            created_at: row.created_at,
        }
    }
}

fn reaction_emoji_from_str(s: &str) -> ReactionEmoji {
    match s {
        "+1" => ReactionEmoji::ThumbsUp,
        "-1" => ReactionEmoji::ThumbsDown,
        "eyes" => ReactionEmoji::Eyes,
        "warning" => ReactionEmoji::Warning,
        "check" => ReactionEmoji::Check,
        "fire" => ReactionEmoji::Fire,
        _ => unreachable!("unexpected reaction_emoji value from database: {}", s),
    }
}

#[derive(sqlx::FromRow)]
struct ReactionWithUsernameRow {
    entry_id: Uuid,
    user_id: Uuid,
    username: String,
    emoji: String,
}

impl From<ReactionWithUsernameRow> for ReactionWithUsername {
    fn from(row: ReactionWithUsernameRow) -> Self {
        Self {
            entry_id: row.entry_id,
            user_id: row.user_id,
            username: row.username,
            emoji: reaction_emoji_from_str(&row.emoji),
        }
    }
}

fn incident_state_from_str(s: &str) -> IncidentState {
    match s {
        "open" => IncidentState::Open,
        "acknowledged" => IncidentState::Acknowledged,
        "escalated" => IncidentState::Escalated,
        "resolved" => IncidentState::Resolved,
        _ => unreachable!("unexpected incident_state value from database: {}", s),
    }
}

fn incident_state_to_str(state: &IncidentState) -> &'static str {
    match state {
        IncidentState::Open => "open",
        IncidentState::Acknowledged => "acknowledged",
        IncidentState::Escalated => "escalated",
        IncidentState::Resolved => "resolved",
    }
}

fn severity_from_str(s: &str) -> Severity {
    match s {
        "low" => Severity::Low,
        "medium" => Severity::Medium,
        "high" => Severity::High,
        "critical" => Severity::Critical,
        _ => unreachable!("unexpected severity value from database: {}", s),
    }
}

fn severity_to_str(severity: &Severity) -> &'static str {
    match severity {
        Severity::Low => "low",
        Severity::Medium => "medium",
        Severity::High => "high",
        Severity::Critical => "critical",
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_incident(
    executor: impl sqlx::PgExecutor<'_>,
    id: Uuid,
    team_id: Uuid,
    title: &str,
    description: &str,
    severity: &Severity,
    created_by: Uuid,
    release_id: Option<Uuid>,
) -> Result<Incident, AppError> {
    let severity_str = severity_to_str(severity);
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        INSERT INTO incidents (id, team_id, title, description, severity, created_by, release_id)
        VALUES ($1, $2, $3, $4, $5::severity, $6, $7)
        RETURNING
            id, team_id, title, description,
            state::TEXT as "state!",
            severity::TEXT as "severity!",
            created_by, assigned_to, release_id, created_at, resolved_at
        "#,
        id,
        team_id,
        title,
        description,
        severity_str as &str,
        created_by,
        release_id,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to insert incident");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn find_incident_by_id(
    pool: &PgPool,
    incident_id: Uuid,
) -> Result<Option<Incident>, AppError> {
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        SELECT
            id, team_id, title, description,
            state::TEXT as "state!",
            severity::TEXT as "severity!",
            created_by, assigned_to, release_id, created_at, resolved_at
        FROM incidents
        WHERE id = $1
        "#,
        incident_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %incident_id, "failed to find incident by id");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn find_incidents_by_team_id(
    pool: &PgPool,
    team_id: Uuid,
) -> Result<Vec<Incident>, AppError> {
    let rows = sqlx::query_as!(
        IncidentRow,
        r#"
        SELECT
            id, team_id, title, description,
            state::TEXT as "state!",
            severity::TEXT as "severity!",
            created_by, assigned_to, release_id, created_at, resolved_at
        FROM incidents
        WHERE team_id = $1
        ORDER BY created_at DESC
        "#,
        team_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to find incidents by team id");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn update_incident_state(
    executor: impl sqlx::PgExecutor<'_>,
    incident_id: Uuid,
    new_state: &IncidentState,
    resolved_at: Option<OffsetDateTime>,
) -> Result<Incident, AppError> {
    let state_str = incident_state_to_str(new_state);
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        UPDATE incidents
        SET state = $2::incident_state, resolved_at = $3
        WHERE id = $1
        RETURNING
            id, team_id, title, description,
            state::TEXT as "state!",
            severity::TEXT as "severity!",
            created_by, assigned_to, release_id, created_at, resolved_at
        "#,
        incident_id,
        state_str as &str,
        resolved_at,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %incident_id, "failed to update incident state");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn update_incident_severity(
    executor: impl sqlx::PgExecutor<'_>,
    incident_id: Uuid,
    new_severity: &Severity,
) -> Result<Incident, AppError> {
    let severity_str = severity_to_str(new_severity);
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        UPDATE incidents
        SET severity = $2::severity
        WHERE id = $1
        RETURNING
            id, team_id, title, description,
            state::TEXT as "state!",
            severity::TEXT as "severity!",
            created_by, assigned_to, release_id, created_at, resolved_at
        "#,
        incident_id,
        severity_str as &str,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %incident_id, "failed to update incident severity");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn update_incident_assigned_to(
    executor: impl sqlx::PgExecutor<'_>,
    incident_id: Uuid,
    assigned_to: Option<Uuid>,
) -> Result<Incident, AppError> {
    let row = sqlx::query_as!(
        IncidentRow,
        r#"
        UPDATE incidents
        SET assigned_to = $2
        WHERE id = $1
        RETURNING
            id, team_id, title, description,
            state::TEXT as "state!",
            severity::TEXT as "severity!",
            created_by, assigned_to, release_id, created_at, resolved_at
        "#,
        incident_id,
        assigned_to,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %incident_id, "failed to update incident assigned_to");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn has_unresolved_incidents_for_release(
    executor: impl sqlx::PgExecutor<'_>,
    release_id: Uuid,
) -> Result<bool, AppError> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM incidents
        WHERE release_id = $1 AND state != 'resolved'::incident_state
        "#,
        release_id,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %release_id, "failed to count unresolved incidents for release");
        AppError::InternalError
    })?;

    Ok(count.unwrap_or(0) > 0)
}

pub async fn insert_timeline_entry(
    executor: impl sqlx::PgExecutor<'_>,
    id: Uuid,
    incident_id: Uuid,
    author_id: Uuid,
    content: &str,
) -> Result<TimelineEntry, AppError> {
    let row = sqlx::query_as!(
        TimelineEntryRow,
        r#"
        INSERT INTO timeline_entries (id, incident_id, author_id, content)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        id,
        incident_id,
        author_id,
        content,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %incident_id, "failed to insert timeline entry");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn find_timeline_entries_by_incident_id(
    pool: &PgPool,
    incident_id: Uuid,
) -> Result<Vec<TimelineEntry>, AppError> {
    let rows = sqlx::query_as!(
        TimelineEntryRow,
        r#"
        SELECT *
        FROM timeline_entries
        WHERE incident_id = $1
        ORDER BY created_at ASC
        "#,
        incident_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %incident_id, "failed to find timeline entries");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn find_timeline_entry_by_id(
    pool: &PgPool,
    entry_id: Uuid,
) -> Result<Option<TimelineEntry>, AppError> {
    let row = sqlx::query_as!(
        TimelineEntryRow,
        r#"SELECT * FROM timeline_entries WHERE id = $1"#,
        entry_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %entry_id, "failed to find timeline entry by id");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn update_timeline_entry_content(
    executor: impl sqlx::PgExecutor<'_>,
    entry_id: Uuid,
    new_content: &str,
) -> Result<TimelineEntry, AppError> {
    let row = sqlx::query_as!(
        TimelineEntryRow,
        r#"
        UPDATE timeline_entries
        SET content = $2, edited_at = now()
        WHERE id = $1
        RETURNING *
        "#,
        entry_id,
        new_content,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %entry_id, "failed to update timeline entry content");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn insert_reaction(
    pool: &PgPool,
    id: Uuid,
    entry_id: Uuid,
    user_id: Uuid,
    emoji: &ReactionEmoji,
) -> Result<Reaction, AppError> {
    let emoji_str = emoji.as_str();
    let row = sqlx::query_as!(
        ReactionRow,
        r#"
        INSERT INTO timeline_reactions (id, entry_id, user_id, emoji)
        VALUES ($1, $2, $3, $4::reaction_emoji)
        RETURNING id, entry_id, user_id, emoji::TEXT as "emoji!", created_at
        "#,
        id,
        entry_id,
        user_id,
        emoji_str as &str,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db)
            if db.constraint() == Some("timeline_reactions_unique_reaction") =>
        {
            AppError::Conflict("You already reacted with this emoji".into())
        }
        e => {
            tracing::error!(error = ?e, %entry_id, %user_id, "failed to insert reaction");
            AppError::InternalError
        }
    })?;

    Ok(row.into())
}

pub async fn delete_reaction(
    pool: &PgPool,
    entry_id: Uuid,
    user_id: Uuid,
    emoji: &ReactionEmoji,
) -> Result<(), AppError> {
    let emoji_str = emoji.as_str();
    sqlx::query!(
        r#"
        DELETE FROM timeline_reactions
        WHERE entry_id = $1 AND user_id = $2 AND emoji = $3::reaction_emoji
        "#,
        entry_id,
        user_id,
        emoji_str as &str,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %entry_id, %user_id, "failed to delete reaction");
        AppError::InternalError
    })?;

    Ok(())
}

pub async fn find_reactions_by_incident_id(
    pool: &PgPool,
    incident_id: Uuid,
) -> Result<Vec<ReactionWithUsername>, AppError> {
    let rows = sqlx::query_as!(
        ReactionWithUsernameRow,
        r#"
        SELECT tr.entry_id, tr.user_id, u.username, tr.emoji::TEXT as "emoji!"
        FROM timeline_reactions tr
        JOIN timeline_entries te ON te.id = tr.entry_id
        JOIN users u ON u.id = tr.user_id
        WHERE te.incident_id = $1
        ORDER BY tr.created_at ASC
        "#,
        incident_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %incident_id, "failed to find reactions by incident id");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}
