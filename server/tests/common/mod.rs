#![allow(dead_code)]

use axum::Router;
use axum::body::Body;
use axum::http::Request;
use sqlx::PgPool;
use tower::ServiceExt;

use vigil_server::AppState;
use vigil_server::features::auth::{
    model::{Session, User},
    service,
};
use vigil_server::features::teams::model::{Team, TeamMember};
use vigil_server::shared::config::Config;

pub fn build_test_app(pool: PgPool) -> Router {
    let config = Config::from_env();
    let state = AppState::new(config, pool);
    vigil_server::build_router(state)
}

pub async fn create_test_user(pool: &PgPool) -> (User, Session) {
    service::register(pool, "alissa", "alissa@example.com", "password123")
        .await
        .expect("failed to create test user")
}

pub async fn create_test_user_with(pool: &PgPool, username: &str, email: &str) -> (User, Session) {
    service::register(pool, username, email, "password123")
        .await
        .expect("failed to create test user")
}

pub async fn create_test_team(
    pool: &PgPool,
    name: &str,
    creator_id: uuid::Uuid,
) -> (Team, TeamMember) {
    vigil_server::features::teams::service::create_team(pool, name, creator_id)
        .await
        .expect("failed to create test team")
}

pub async fn post_json(
    app: Router,
    uri: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    app.oneshot(request).await.unwrap()
}

pub async fn post_json_with_token(
    app: Router,
    uri: &str,
    token: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    app.oneshot(request).await.unwrap()
}

pub async fn get_with_token(app: Router, uri: &str, token: &str) -> axum::response::Response {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    app.oneshot(request).await.unwrap()
}

pub async fn post_with_token(app: Router, uri: &str, token: &str) -> axum::response::Response {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    app.oneshot(request).await.unwrap()
}
