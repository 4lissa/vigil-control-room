use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::auth::{model::{Session, User}, repo};
use crate::shared::error::AppError;
use sqlx::PgPool;

const SESSION_DURATION_DAYS: i64 = 7;

pub async fn register(
    pool:     &PgPool,
    username: &str,
    email:    &str,
    password: &str,
) -> Result<User, AppError> {
    let password_hash = hash_password(password).await?;

    let user = repo::insert_user(
        pool,
        Uuid::now_v7(),
        username,
        email,
        Some(&password_hash),
        None,
    )
    .await?;

    Ok(user)
}

pub async fn login(
    pool:     &PgPool,
    email:    &str,
    password: &str,
) -> Result<(User, Session), AppError> {
    let user = repo::find_user_by_email(pool, email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let hash = user
        .password_hash
        .as_deref()
        .ok_or(AppError::Unauthorized)?;

    verify_password(password, hash).await?;

    repo::delete_expired_sessions(pool, user.id).await?;

    let session = repo::insert_session(
        pool,
        Uuid::now_v7(),
        user.id,
        OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS),
    )
    .await?;

    Ok((user, session))
}

pub async fn logout(pool: &PgPool, session_id: Uuid) -> Result<(), AppError> {
    repo::delete_session(pool, session_id).await?;
    Ok(())
}

pub async fn me(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    repo::find_user_by_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound("user not found".into()))
}

pub async fn update_profile(
    pool:         &PgPool,
    user_id:      Uuid,
    username:     Option<&str>,
    language:     Option<&str>,
    new_password: Option<&str>,
    old_password: Option<&str>,
) -> Result<User, AppError> {
    let password_hash = match new_password {
        Some(new_pwd) => {
            let user = repo::find_user_by_id(pool, user_id)
                .await?
                .ok_or(AppError::NotFound("user not found".into()))?;

            let old_pwd = old_password.ok_or(AppError::BadRequest(
                "current password is required to set a new one".into(),
            ))?;

            let hash = user
                .password_hash
                .as_deref()
                .ok_or(AppError::BadRequest(
                    "cannot set password on an OAuth-only account".into(),
                ))?;

            verify_password(old_pwd, hash).await?;

            Some(hash_password(new_pwd).await?)
        }
        None => None,
    };

    let user = repo::update_user(
        pool,
        user_id,
        username,
        language,
        password_hash.as_deref(),
    )
    .await?;

    Ok(user)
}

async fn hash_password(password: &str) -> Result<String, AppError> {
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                tracing::error!(error = ?e, "failed to hash password");
                AppError::InternalError
            })?;
        Ok(hash.to_string())
    })
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, "password hashing task panicked");
        AppError::InternalError
    })?
}

async fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    let password = password.to_owned();
    let hash = hash.to_owned();
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash).map_err(|e| {
            tracing::error!(error = ?e, "failed to parse password hash");
            AppError::InternalError
        })?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)
    })
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, "password verification task panicked");
        AppError::InternalError
    })?
}
