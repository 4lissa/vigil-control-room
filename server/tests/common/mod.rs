use sqlx::PgPool;
use uuid::Uuid;

use vigil_server::features::auth::{model::User, repo, service};

pub async fn create_test_user(pool: &PgPool) -> User {
    service::register(pool, "testuser", "test@example.com", "password123")
        .await
        .expect("failed to create test user")
}

pub async fn create_test_user_with(pool: &PgPool, username: &str, email: &str) -> User {
    service::register(pool, username, email, "password123")
        .await
        .expect("failed to create test user")
}

pub async fn create_test_session(
    pool: &PgPool,
    user_id: Uuid,
) -> vigil_server::features::auth::model::Session {
    repo::insert_session(
        pool,
        Uuid::now_v7(),
        user_id,
        time::OffsetDateTime::now_utc() + time::Duration::days(7),
    )
    .await
    .expect("failed to create test session")
}
