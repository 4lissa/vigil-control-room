use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::teams::model::{Role, Team, TeamBan, TeamMember};
use crate::shared::error::AppError;

#[derive(sqlx::FromRow)]
struct TeamRow {
    id: Uuid,
    name: String,
    invitation_code: Option<String>,
    created_by: Option<Uuid>,
    created_at: OffsetDateTime,
}

impl From<TeamRow> for Team {
    fn from(row: TeamRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            invitation_code: row.invitation_code,
            created_by: row.created_by,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TeamMemberRow {
    id: Uuid,
    team_id: Uuid,
    user_id: Uuid,
    role: String,
    joined_at: OffsetDateTime,
}

impl From<TeamMemberRow> for TeamMember {
    fn from(row: TeamMemberRow) -> Self {
        Self {
            id: row.id,
            team_id: row.team_id,
            user_id: row.user_id,
            role: role_from_str(&row.role),
            joined_at: row.joined_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TeamBanRow {
    id: Uuid,
    team_id: Uuid,
    user_id: Uuid,
    banned_by: Option<Uuid>,
    until: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
}

impl From<TeamBanRow> for TeamBan {
    fn from(row: TeamBanRow) -> Self {
        Self {
            id: row.id,
            team_id: row.team_id,
            user_id: row.user_id,
            banned_by: row.banned_by,
            until: row.until,
            created_at: row.created_at,
        }
    }
}

fn role_from_str(s: &str) -> Role {
    match s {
        "manager" => Role::Manager,
        "responder" => Role::Responder,
        "observer" => Role::Observer,
        _ => unreachable!("unexpected role value from database: {}", s),
    }
}

fn role_to_str(role: &Role) -> &'static str {
    match role {
        Role::Manager => "manager",
        Role::Responder => "responder",
        Role::Observer => "observer",
    }
}

pub async fn insert_team(
    executor: impl sqlx::PgExecutor<'_>,
    id: Uuid,
    name: &str,
    created_by: Uuid,
) -> Result<Team, AppError> {
    let row = sqlx::query_as!(
        TeamRow,
        r#"
        INSERT INTO teams (id, name, created_by)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        id,
        name,
        created_by,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %name, "failed to insert team");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn find_team_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Team>, AppError> {
    let row = sqlx::query_as!(TeamRow, r#"SELECT * FROM teams WHERE id = $1"#, id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, %id, "failed to find team by id");
            AppError::InternalError
        })?;

    Ok(row.map(Into::into))
}

pub async fn find_team_by_invitation_code(
    pool: &PgPool,
    code: &str,
) -> Result<Option<Team>, AppError> {
    let row = sqlx::query_as!(
        TeamRow,
        r#"SELECT * FROM teams WHERE invitation_code = $1"#,
        code
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, "failed to find team by invitation code");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn find_teams_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Vec<Team>, AppError> {
    let rows = sqlx::query_as!(
        TeamRow,
        r#"
        SELECT t.*
        FROM teams t
        INNER JOIN team_members tm ON tm.team_id = t.id
        WHERE tm.user_id = $1
        ORDER BY t.created_at ASC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %user_id, "failed to find teams by user id");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn update_invitation_code(
    pool: &PgPool,
    team_id: Uuid,
    new_code: &str,
) -> Result<Team, AppError> {
    let row = sqlx::query_as!(
        TeamRow,
        r#"
        UPDATE teams
        SET invitation_code = $2
        WHERE id = $1
        RETURNING *
        "#,
        team_id,
        new_code,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to update invitation code");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn insert_team_member(
    executor: impl sqlx::PgExecutor<'_>,
    id: Uuid,
    team_id: Uuid,
    user_id: Uuid,
    role: &Role,
) -> Result<TeamMember, AppError> {
    let role_str = role_to_str(role);
    let row = sqlx::query_as!(
        TeamMemberRow,
        r#"
        INSERT INTO team_members (id, team_id, user_id, role)
        VALUES ($1, $2, $3, $4::team_role)
        RETURNING id, team_id, user_id, role::TEXT as "role!", joined_at
        "#,
        id,
        team_id,
        user_id,
        role_str as &str,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint() == Some("team_members_unique_membership") => {
            AppError::Conflict("Already a member of this team".into())
        }
        e => {
            tracing::error!(error = ?e, %team_id, %user_id, "failed to insert team member");
            AppError::InternalError
        }
    })?;

    Ok(row.into())
}

pub async fn find_member(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<Option<TeamMember>, AppError> {
    let row = sqlx::query_as!(
        TeamMemberRow,
        r#"
        SELECT id, team_id, user_id, role::TEXT as "role!", joined_at
        FROM team_members
        WHERE team_id = $1 AND user_id = $2
        "#,
        team_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, %user_id, "failed to find member");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn find_members_by_team_id(
    pool: &PgPool,
    team_id: Uuid,
) -> Result<Vec<TeamMember>, AppError> {
    let rows = sqlx::query_as!(
        TeamMemberRow,
        r#"
        SELECT id, team_id, user_id, role::TEXT as "role!", joined_at
        FROM team_members
        WHERE team_id = $1
        ORDER BY joined_at ASC
        "#,
        team_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, "failed to find members by team id");
        AppError::InternalError
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn update_member_role(
    executor: impl sqlx::PgExecutor<'_>,
    team_id: Uuid,
    user_id: Uuid,
    new_role: &Role,
) -> Result<TeamMember, AppError> {
    let role_str = role_to_str(new_role);
    let row = sqlx::query_as!(
        TeamMemberRow,
        r#"
        UPDATE team_members
        SET role = $3::team_role
        WHERE team_id = $1 AND user_id = $2
        RETURNING id, team_id, user_id, role::TEXT as "role!", joined_at
        "#,
        team_id,
        user_id,
        role_str as &str,
    )
    .fetch_one(executor)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, %user_id, "failed to update member role");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn delete_member(pool: &PgPool, team_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"DELETE FROM team_members WHERE team_id = $1 AND user_id = $2"#,
        team_id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, %user_id, "failed to delete member");
        AppError::InternalError
    })?;

    Ok(())
}

pub async fn insert_ban(
    pool: &PgPool,
    id: Uuid,
    team_id: Uuid,
    user_id: Uuid,
    banned_by: Uuid,
    until: Option<OffsetDateTime>,
) -> Result<TeamBan, AppError> {
    let row = sqlx::query_as!(
        TeamBanRow,
        r#"
        INSERT INTO team_bans (id, team_id, user_id, banned_by, until)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (team_id, user_id) DO UPDATE
            SET banned_by  = EXCLUDED.banned_by,
                until      = EXCLUDED.until,
                created_at = now()
        RETURNING *
        "#,
        id,
        team_id,
        user_id,
        banned_by,
        until,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, %user_id, "failed to insert ban");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn find_active_ban(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<Option<TeamBan>, AppError> {
    let row = sqlx::query_as!(
        TeamBanRow,
        r#"
        SELECT *
        FROM team_bans
        WHERE team_id = $1
          AND user_id = $2
          AND (until IS NULL OR until > now())
        "#,
        team_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, %user_id, "failed to find active ban");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn delete_ban(pool: &PgPool, team_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"DELETE FROM team_bans WHERE team_id = $1 AND user_id = $2"#,
        team_id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %team_id, %user_id, "failed to delete ban");
        AppError::InternalError
    })?;

    Ok(())
}
