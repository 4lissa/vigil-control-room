mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn create_release_returns_201_for_manager(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/releases", team.id),
        &token,
        json!({ "name": "v1.0.0", "steps": ["build", "staging"] }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["name"], "v1.0.0");
    assert_eq!(body["state"], "created");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_release_returns_403_for_observer(pool: PgPool) {
    let (user1, session1) = common::create_test_user(&pool).await;
    let (_, session2) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;

    let app = common::build_test_app(pool.clone());
    let invite = common::post_with_token(
        app,
        &format!("/teams/{}/invite", team.id),
        &session1.id.to_string(),
    )
    .await;
    let body = to_bytes(invite.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let code = body["invitation_code"].as_str().unwrap().to_string();

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        "/teams/join",
        &session2.id.to_string(),
        json!({ "code": code }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/releases", team.id),
        &session2.id.to_string(),
        json!({ "name": "v1.0.0", "steps": ["build"] }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_releases_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        &format!("/teams/{}/releases", team.id),
        &token,
        json!({ "name": "v1.0.0", "steps": ["build"] }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response =
        common::get_with_token(app, &format!("/teams/{}/releases", team.id), &token).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_release_steps_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/releases", team.id),
        &token,
        json!({ "name": "v1.0.0", "steps": ["build", "staging"] }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let release_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::get_with_token(
        app,
        &format!("/teams/{}/releases/{}/steps", team.id, release_id),
        &token,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let steps = body.as_array().unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0]["name"], "build");
}

#[sqlx::test(migrations = "./migrations")]
async fn validate_step_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/releases", team.id),
        &token,
        json!({ "name": "v1.0.0", "steps": ["build"] }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let release_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool.clone());
    let res = common::get_with_token(
        app,
        &format!("/teams/{}/releases/{}/steps", team.id, release_id),
        &token,
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let step_id = body[0]["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::post_with_token(
        app,
        &format!(
            "/teams/{}/releases/{}/steps/{}/validate",
            team.id, release_id, step_id
        ),
        &token,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body["validated_at"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn cancel_release_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/releases", team.id),
        &token,
        json!({ "name": "v1.0.0", "steps": ["build"] }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let release_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::post_with_token(
        app,
        &format!("/teams/{}/releases/{}/cancel", team.id, release_id),
        &token,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["state"], "cancelled");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_incident_with_release_id_returns_201(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/releases", team.id),
        &token,
        json!({ "name": "v1.0.0", "steps": ["build"] }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let release_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high", "release_id": release_id }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["release_id"], release_id);
}
