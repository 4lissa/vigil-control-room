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

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message = body["error"]["message"].as_str().unwrap();
    assert_eq!(message, "Email must be a valid email address");
    assert!(!message.contains("not-an-email"));
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
async fn github_redirect_sets_cookie_and_redirects_to_github(pool: PgPool) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = common::build_test_app(pool);
    let request = Request::builder()
        .method("GET")
        .uri("/github")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.starts_with("https://github.com/login/oauth/authorize?"));

    let set_cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(set_cookie.starts_with("oauth_state="));
    assert!(set_cookie.contains("HttpOnly"));
}

#[sqlx::test(migrations = "./migrations")]
async fn github_callback_redirects_to_login_with_error_without_cookie(pool: PgPool) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = common::build_test_app(pool);
    let request = Request::builder()
        .method("GET")
        .uri("/github/callback?code=fakecode&state=fakestate")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.ends_with("/login?error=oauth_failed"));
}

#[sqlx::test(migrations = "./migrations")]
async fn github_callback_redirects_to_login_with_error_when_state_does_not_match_cookie(
    pool: PgPool,
) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = common::build_test_app(pool);
    let request = Request::builder()
        .method("GET")
        .uri("/github/callback?code=fakecode&state=wrong-state")
        .header("Cookie", "oauth_state=correct-state")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.ends_with("/login?error=oauth_failed"));
}

#[sqlx::test(migrations = "./migrations")]
async fn github_callback_redirects_to_login_with_error_without_code(pool: PgPool) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = common::build_test_app(pool);
    let request = Request::builder()
        .method("GET")
        .uri("/github/callback?state=matching-state")
        .header("Cookie", "oauth_state=matching-state")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.ends_with("/login?error=oauth_failed"));
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

#[sqlx::test(migrations = "./migrations")]
async fn connect_service_returns_204_for_http(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        "/connected-services/http",
        &session.id.to_string(),
        json!({ "token": "my-token" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn connect_service_returns_422_for_github(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        "/connected-services/github",
        &session.id.to_string(),
        json!({ "token": "my-token" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn connect_service_returns_401_without_token(pool: PgPool) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let app = common::build_test_app(pool);
    let request = Request::builder()
        .method("POST")
        .uri("/connected-services/http")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"token":"my-token"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_connected_service_status_returns_false_initially(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;

    let app = common::build_test_app(pool);
    let response =
        common::get_with_token(app, "/connected-services/http", &session.id.to_string()).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["connected"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_connected_service_status_returns_true_after_connecting(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        "/connected-services/http",
        &token,
        json!({ "token": "my-token" }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, "/connected-services/http", &token).await;

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["connected"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn disconnect_service_returns_204_and_clears_the_status(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        "/connected-services/http",
        &token,
        json!({ "token": "my-token" }),
    )
    .await;

    let app = common::build_test_app(pool.clone());
    let response = common::delete_with_token(app, "/connected-services/http", &token).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, "/connected-services/http", &token).await;
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["connected"], false);
}
