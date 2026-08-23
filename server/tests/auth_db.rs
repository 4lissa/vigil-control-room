mod common;

use sqlx::PgPool;
use vigil_server::features::auth::model::ServiceKind;
use vigil_server::features::auth::repo;
use vigil_server::features::auth::service;
use vigil_server::shared::config::Config;
use vigil_server::shared::error::AppError;

#[sqlx::test(migrations = "./migrations")]
async fn register_succeeds_with_valid_input(pool: PgPool) {
    let (user, session) = service::register(&pool, "alissa", "alissa@example.com", "password123")
        .await
        .unwrap();

    assert_eq!(user.username, "alissa");
    assert_eq!(user.email, "alissa@example.com");
    assert!(user.password_hash.is_some());
    assert!(user.github_id.is_none());
    assert_eq!(session.user_id, user.id);
    assert!(!session.is_expired());
}

#[sqlx::test(migrations = "./migrations")]
async fn register_fails_when_email_already_taken(pool: PgPool) {
    service::register(&pool, "alissa", "alissa@example.com", "password123")
        .await
        .unwrap();

    let result = service::register(&pool, "bob", "alissa@example.com", "password123").await;
    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn register_fails_when_username_already_taken(pool: PgPool) {
    service::register(&pool, "alissa", "alissa@example.com", "password123")
        .await
        .unwrap();

    let result = service::register(&pool, "alissa", "bob@example.com", "password123").await;
    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn login_succeeds_with_correct_credentials(pool: PgPool) {
    service::register(&pool, "alissa", "alissa@example.com", "password123")
        .await
        .unwrap();

    let (user, session) = service::login(&pool, "alissa@example.com", "password123")
        .await
        .unwrap();

    assert_eq!(user.email, "alissa@example.com");
    assert_eq!(session.user_id, user.id);
    assert!(!session.is_expired());
}

#[sqlx::test(migrations = "./migrations")]
async fn login_fails_with_wrong_password(pool: PgPool) {
    service::register(&pool, "alissa", "alissa@example.com", "password123")
        .await
        .unwrap();

    let result = service::login(&pool, "alissa@example.com", "wrongpassword").await;
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[sqlx::test(migrations = "./migrations")]
async fn login_fails_when_user_not_found(pool: PgPool) {
    let result = service::login(&pool, "nobody@example.com", "password123").await;
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[sqlx::test(migrations = "./migrations")]
async fn logout_invalidates_session(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;

    service::logout(&pool, session.id).await.unwrap();

    let result = vigil_server::features::auth::repo::find_session_by_id(&pool, session.id)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn me_returns_user_when_found(pool: PgPool) {
    let (created, _) = common::create_test_user(&pool).await;
    let user = service::me(&pool, created.id).await.unwrap();
    assert_eq!(user.id, created.id);
}

#[sqlx::test(migrations = "./migrations")]
async fn me_returns_not_found_when_user_missing(pool: PgPool) {
    let result = service::me(&pool, uuid::Uuid::now_v7()).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_changes_username(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let updated = service::update_profile(&pool, user.id, Some("newname"), None, None, None)
        .await
        .unwrap();
    assert_eq!(updated.username, "newname");
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_changes_language(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let updated = service::update_profile(&pool, user.id, None, Some("fr"), None, None)
        .await
        .unwrap();
    assert_eq!(updated.language, "fr");
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_changes_password(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    service::update_profile(
        &pool,
        user.id,
        None,
        None,
        Some("newpassword123"),
        Some("password123"),
    )
    .await
    .unwrap();

    let result = service::login(&pool, "alissa@example.com", "newpassword123").await;
    assert!(result.is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_fails_with_wrong_old_password(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let result = service::update_profile(
        &pool,
        user.id,
        None,
        None,
        Some("newpassword123"),
        Some("wrongpassword"),
    )
    .await;
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[sqlx::test(migrations = "./migrations")]
async fn update_profile_fails_when_username_already_taken(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    common::create_test_user_with(&pool, "bob", "bob@example.com").await;

    let result = service::update_profile(&pool, user.id, Some("bob"), None, None, None).await;
    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn connect_service_stores_an_encrypted_token(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let config = Config::from_env();

    service::connect_service(&pool, &config, user.id, "http", "my-token")
        .await
        .unwrap();

    let connected = repo::find_connected_service(&pool, user.id, ServiceKind::Http)
        .await
        .unwrap()
        .unwrap();

    assert_ne!(connected.encrypted_token, b"my-token");
}

#[sqlx::test(migrations = "./migrations")]
async fn connect_service_rejects_github(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let config = Config::from_env();

    let result = service::connect_service(&pool, &config, user.id, "github", "my-token").await;

    assert!(matches!(result, Err(AppError::ValidationError(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn connect_service_rejects_unknown_service(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let config = Config::from_env();

    let result = service::connect_service(&pool, &config, user.id, "discord", "my-token").await;

    assert!(matches!(result, Err(AppError::ValidationError(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn connect_service_replaces_an_existing_token(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let config = Config::from_env();

    service::connect_service(&pool, &config, user.id, "http", "old-token")
        .await
        .unwrap();
    service::connect_service(&pool, &config, user.id, "http", "new-token")
        .await
        .unwrap();

    let connected = repo::find_connected_service(&pool, user.id, ServiceKind::Http)
        .await
        .unwrap()
        .unwrap();
    let decrypted = vigil_server::shared::crypto::decrypt(
        &config.token_encryption_key,
        &connected.encrypted_token,
    )
    .unwrap();

    assert_eq!(decrypted, "new-token");
}

#[sqlx::test(migrations = "./migrations")]
async fn is_service_connected_returns_false_when_nothing_connected(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;

    let connected = service::is_service_connected(&pool, user.id, "http")
        .await
        .unwrap();

    assert!(!connected);
}

#[sqlx::test(migrations = "./migrations")]
async fn is_service_connected_returns_true_after_connecting(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let config = Config::from_env();

    service::connect_service(&pool, &config, user.id, "http", "my-token")
        .await
        .unwrap();

    let connected = service::is_service_connected(&pool, user.id, "http")
        .await
        .unwrap();

    assert!(connected);
}

#[sqlx::test(migrations = "./migrations")]
async fn disconnect_service_removes_the_connection(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let config = Config::from_env();

    service::connect_service(&pool, &config, user.id, "http", "my-token")
        .await
        .unwrap();
    service::disconnect_service(&pool, user.id, "http")
        .await
        .unwrap();

    let connected = repo::find_connected_service(&pool, user.id, ServiceKind::Http)
        .await
        .unwrap();

    assert!(connected.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn disconnect_service_rejects_github(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;

    let result = service::disconnect_service(&pool, user.id, "github").await;

    assert!(matches!(result, Err(AppError::ValidationError(_))));
}
