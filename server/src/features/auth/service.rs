use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::auth::{
    model::{ServiceKind, Session, User},
    repo,
};
use crate::shared::clients::github as github_client;
use crate::shared::config::Config;
use crate::shared::crypto;
use crate::shared::error::AppError;
use sqlx::PgPool;

const SESSION_DURATION_DAYS: i64 = 7;

pub async fn register(
    pool: &PgPool,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(User, Session), AppError> {
    let password_hash = hash_password(password).await?;

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    let user = repo::insert_user(
        &mut *tx,
        Uuid::now_v7(),
        username,
        email,
        Some(&password_hash),
        None,
    )
    .await?;

    let session = repo::insert_session(
        &mut *tx,
        Uuid::now_v7(),
        user.id,
        OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS),
    )
    .await?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok((user, session))
}

pub async fn login(
    pool: &PgPool,
    email: &str,
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

pub fn github_authorize_url(config: &Config) -> (String, String) {
    github_client::authorize_url(&config.github_client_id, &config.github_redirect_uri)
}

pub async fn handle_github_callback(
    pool: &PgPool,
    http_client: &reqwest::Client,
    config: &Config,
    code: &str,
) -> Result<(User, Session), AppError> {
    let access_token = github_client::exchange_code_for_token(
        http_client,
        &config.github_client_id,
        &config.github_client_secret,
        &config.github_redirect_uri,
        code,
    )
    .await?;
    let profile = github_client::fetch_profile(http_client, &access_token).await?;
    let github_id = profile.id.to_string();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    let user = match repo::find_user_by_github_id(&mut *tx, &github_id).await? {
        Some(user) => user,
        None => create_user_from_github(&mut tx, &github_id, &profile).await?,
    };

    let encrypted_token = crypto::encrypt(&config.token_encryption_key, &access_token);
    repo::upsert_connected_service(
        &mut *tx,
        Uuid::now_v7(),
        user.id,
        ServiceKind::Github,
        &encrypted_token,
    )
    .await?;

    repo::delete_expired_sessions(&mut *tx, user.id).await?;
    let session = repo::insert_session(
        &mut *tx,
        Uuid::now_v7(),
        user.id,
        OffsetDateTime::now_utc() + time::Duration::days(SESSION_DURATION_DAYS),
    )
    .await?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok((user, session))
}

async fn create_user_from_github(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    github_id: &str,
    profile: &github_client::UserProfile,
) -> Result<User, AppError> {
    let email = profile.email.clone().ok_or_else(|| {
        AppError::BadRequest("GitHub account has no accessible email address".into())
    })?;

    match repo::insert_user(
        &mut **tx,
        Uuid::now_v7(),
        &profile.login,
        &email,
        None,
        Some(github_id),
    )
    .await
    {
        Ok(user) => Ok(user),
        Err(AppError::Conflict(_)) => {
            let username = format!("{}-{}", profile.login, &Uuid::now_v7().to_string()[..8]);
            repo::insert_user(
                &mut **tx,
                Uuid::now_v7(),
                &username,
                &email,
                None,
                Some(github_id),
            )
            .await
        }
        Err(e) => Err(e),
    }
}

pub async fn logout(pool: &PgPool, session_id: Uuid) -> Result<(), AppError> {
    repo::delete_session(pool, session_id).await?;
    Ok(())
}

pub async fn me(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    repo::find_user_by_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".into()))
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    username: Option<&str>,
    language: Option<&str>,
    new_password: Option<&str>,
    old_password: Option<&str>,
) -> Result<User, AppError> {
    let password_hash = match new_password {
        Some(new_pwd) => {
            let user = repo::find_user_by_id(pool, user_id)
                .await?
                .ok_or(AppError::NotFound("User not found".into()))?;

            let old_pwd = old_password.ok_or(AppError::BadRequest(
                "Current password is required to set a new one".into(),
            ))?;

            let hash = user.password_hash.as_deref().ok_or(AppError::BadRequest(
                "Cannot set password on an OAuth-only account".into(),
            ))?;

            verify_password(old_pwd, hash).await?;

            Some(hash_password(new_pwd).await?)
        }
        None => None,
    };

    let user =
        repo::update_user(pool, user_id, username, language, password_hash.as_deref()).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn test_config() -> Config {
        Config {
            database_url: String::new(),
            server_port: 8080,
            cors_origin: HeaderValue::from_static("http://localhost:3000"),
            github_client_id: "client123".into(),
            github_client_secret: "secret123".into(),
            github_redirect_uri: "http://localhost:8080/github/callback".into(),
            token_encryption_key: [0u8; 32],
            kickoff_token_hash: String::new(),
        }
    }

    #[test]
    fn github_authorize_url_includes_client_id_and_redirect_uri() {
        let (url, _state) = github_authorize_url(&test_config());

        assert!(url.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(url.contains("client_id=client123"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fgithub%2Fcallback"));
    }

    #[test]
    fn github_authorize_url_state_is_reflected_in_the_url() {
        let (url, state) = github_authorize_url(&test_config());
        assert!(url.contains(&format!("state={state}")));
    }

    #[test]
    fn github_authorize_url_generates_a_distinct_state_each_call() {
        let (_, state1) = github_authorize_url(&test_config());
        let (_, state2) = github_authorize_url(&test_config());
        assert_ne!(state1, state2);
    }

    #[tokio::test]
    async fn hash_then_verify_with_correct_password_succeeds() {
        let hash = hash_password("password123").await.unwrap();
        assert!(verify_password("password123", &hash).await.is_ok());
    }

    #[tokio::test]
    async fn verify_with_wrong_password_returns_unauthorized() {
        let hash = hash_password("password123").await.unwrap();
        let result = verify_password("wrong-password", &hash).await;
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[tokio::test]
    async fn verify_with_malformed_hash_returns_internal_error() {
        let result = verify_password("password123", "not-a-valid-hash").await;
        assert!(matches!(result, Err(AppError::InternalError)));
    }

    #[tokio::test]
    async fn hash_password_produces_distinct_hashes_for_same_password() {
        let hash1 = hash_password("password123").await.unwrap();
        let hash2 = hash_password("password123").await.unwrap();
        assert_ne!(hash1, hash2, "salts should differ between hashes");
    }
}
