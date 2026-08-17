mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

async fn make_teammates(pool: &PgPool) -> (uuid::Uuid, String, uuid::Uuid, String) {
    let (user1, session1) = common::create_test_user(pool).await;
    let (user2, session2) = common::create_test_user_with(pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(pool, "Ops", user1.id).await;

    let code = vigil_server::features::teams::service::generate_invitation_code_for_team(
        pool, team.id, user1.id,
    )
    .await
    .unwrap();
    vigil_server::features::teams::service::join_team(pool, &code, user2.id)
        .await
        .unwrap();

    (
        user1.id,
        session1.id.to_string(),
        user2.id,
        session2.id.to_string(),
    )
}

#[sqlx::test(migrations = "./migrations")]
async fn send_message_returns_201_for_teammate(pool: PgPool) {
    let (_, token1, user2_id, _) = make_teammates(&pool).await;

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/messages/{}", user2_id),
        &token1,
        json!({ "content": "Hey, got a minute?" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["content"], "Hey, got a minute?");
}

#[sqlx::test(migrations = "./migrations")]
async fn send_message_returns_403_without_shared_team(pool: PgPool) {
    let (_, session1) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/messages/{}", user2.id),
        &session1.id.to_string(),
        json!({ "content": "Hey" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn send_message_returns_404_for_unknown_recipient(pool: PgPool) {
    let (_, session1) = common::create_test_user(&pool).await;

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/messages/{}", uuid::Uuid::now_v7()),
        &session1.id.to_string(),
        json!({ "content": "Hey" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_conversation_returns_200(pool: PgPool) {
    let (_, token1, user2_id, _) = make_teammates(&pool).await;

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        &format!("/messages/{}", user2_id),
        &token1,
        json!({ "content": "Hey, got a minute?" }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, &format!("/messages/{}", user2_id), &token1).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_conversation_is_visible_to_the_recipient(pool: PgPool) {
    let (user1_id, token1, user2_id, token2) = make_teammates(&pool).await;

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        &format!("/messages/{}", user2_id),
        &token1,
        json!({ "content": "Hey, got a minute?" }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, &format!("/messages/{}", user1_id), &token2).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_conversations_returns_200(pool: PgPool) {
    let (_, token1, user2_id, _) = make_teammates(&pool).await;

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        &format!("/messages/{}", user2_id),
        &token1,
        json!({ "content": "Hey, got a minute?" }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, "/messages", &token1).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
}
