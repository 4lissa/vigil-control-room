mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn register_returns_201_with_token(pool: PgPool) {
    let app = common::build_test_app(pool);
    let response = common::post_json(
        app,
        "/register",
        json!({
            "username": "alissa",
            "email": "alissa@example.com",
            "password": "password123"
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["username"], "alissa");
}

#[sqlx::test(migrations = "./migrations")]
async fn register_returns_422_on_invalid_email(pool: PgPool) {
    let app = common::build_test_app(pool);
    let response = common::post_json(
        app,
        "/register",
        json!({
            "username": "alissa",
            "email": "not-an-email",
            "password": "password123"
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_returns_200_with_token(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    common::post_json(
        app,
        "/register",
        json!({
            "username": "alissa",
            "email": "alissa@example.com",
            "password": "password123"
        }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::post_json(
        app,
        "/login",
        json!({
            "email": "alissa@example.com",
            "password": "password123"
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body["token"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn me_returns_401_without_token(pool: PgPool) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = common::build_test_app(pool);
    let request = Request::builder()
        .method("GET")
        .uri("/me")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn me_returns_200_with_valid_token(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let register_response = common::post_json(
        app,
        "/register",
        json!({
            "username": "alissa",
            "email": "alissa@example.com",
            "password": "password123"
        }),
    )
    .await;

    let body = to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = body["token"].as_str().unwrap().to_string();

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, "/me", &token).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn logout_returns_204_and_invalidates_token(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let register_response = common::post_json(
        app,
        "/register",
        json!({
            "username": "alissa",
            "email": "alissa@example.com",
            "password": "password123"
        }),
    )
    .await;

    let body = to_bytes(register_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = body["token"].as_str().unwrap().to_string();

    let app = common::build_test_app(pool.clone());
    let response = common::post_with_token(app, "/logout", &token).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, "/me", &token).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
