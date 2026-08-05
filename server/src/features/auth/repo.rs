use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::auth::model::{Session, User};
use crate::shared::error::AppError;

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: Option<String>,
    github_id: Option<String>,
    language: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            github_id: row.github_id,
            language: row.language,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    created_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            created_at: row.created_at,
            expires_at: row.expires_at,
        }
    }
}

pub async fn insert_user(
    pool: &PgPool,
    id: Uuid,
    username: &str,
    email: &str,
    password_hash: Option<&str>,
    github_id: Option<&str>,
) -> Result<User, AppError> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        INSERT INTO users (id, username, email, password_hash, github_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        id,
        username,
        email,
        password_hash,
        github_id,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint() == Some("users_username_key") => {
            AppError::Conflict("Username already taken".into())
        }
        sqlx::Error::Database(db) if db.constraint() == Some("users_email_key") => {
            AppError::Conflict("Email already taken".into())
        }
        e => {
            tracing::error!(error = ?e, %username, %email, "failed to insert user");
            AppError::InternalError
        }
    })?;

    Ok(row.into())
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    let row = sqlx::query_as!(UserRow, r#"SELECT * FROM users WHERE email = $1"#, email)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, %email, "failed to find user by email");
            AppError::InternalError
        })?;

    Ok(row.map(Into::into))
}

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    let row = sqlx::query_as!(UserRow, r#"SELECT * FROM users WHERE id = $1"#, id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, %id, "failed to find user by id");
            AppError::InternalError
        })?;

    Ok(row.map(Into::into))
}

pub async fn find_user_by_github_id(
    pool: &PgPool,
    github_id: &str,
) -> Result<Option<User>, AppError> {
    let row = sqlx::query_as!(
        UserRow,
        r#"SELECT * FROM users WHERE github_id = $1"#,
        github_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %github_id, "failed to find user by github_id");
        AppError::InternalError
    })?;

    Ok(row.map(Into::into))
}

pub async fn update_user(
    pool: &PgPool,
    id: Uuid,
    username: Option<&str>,
    language: Option<&str>,
    password_hash: Option<&str>,
) -> Result<User, AppError> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        UPDATE users
        SET
            username      = COALESCE($2, username),
            language      = COALESCE($3, language),
            password_hash = COALESCE($4, password_hash),
            updated_at    = now()
        WHERE id = $1
        RETURNING *
        "#,
        id,
        username,
        language,
        password_hash,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint() == Some("users_username_key") => {
            AppError::Conflict("Username already taken".into())
        }
        e => {
            tracing::error!(error = ?e, %id, "failed to update user");
            AppError::InternalError
        }
    })?;

    Ok(row.into())
}

pub async fn insert_session(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    expires_at: OffsetDateTime,
) -> Result<Session, AppError> {
    let row = sqlx::query_as!(
        SessionRow,
        r#"
        INSERT INTO sessions (id, user_id, expires_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        id,
        user_id,
        expires_at,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %user_id, "failed to insert session");
        AppError::InternalError
    })?;

    Ok(row.into())
}

pub async fn find_session_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Session>, AppError> {
    let row = sqlx::query_as!(SessionRow, r#"SELECT * FROM sessions WHERE id = $1"#, id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, %id, "failed to find session by id");
            AppError::InternalError
        })?;

    Ok(row.map(Into::into))
}

pub async fn delete_session(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query!(r#"DELETE FROM sessions WHERE id = $1"#, id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, %id, "failed to delete session");
            AppError::InternalError
        })?;

    Ok(())
}

pub async fn delete_expired_sessions(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"DELETE FROM sessions WHERE user_id = $1 AND expires_at < now()"#,
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, %user_id, "failed to delete expired sessions");
        AppError::InternalError
    })?;

    Ok(())
}
