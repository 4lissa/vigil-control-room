mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn create_team_returns_201(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool);
    let response =
        common::post_json_with_token(app, "/teams", &token, json!({ "name": "My Team" })).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["team"]["name"], "My Team");
    assert_eq!(body["member"]["role"], "manager");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_team_returns_401_without_token(pool: PgPool) {
    let app = common::build_test_app(pool);
    let response = common::post_json(app, "/teams", json!({ "name": "My Team" })).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_team_returns_422_on_short_name(pool: PgPool) {
    let (_, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool);
    let response =
        common::post_json_with_token(app, "/teams", &token, json!({ "name": "A" })).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_my_teams_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();
    common::create_test_team(&pool, "My Team", user.id).await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, "/teams", &token).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["name"], "My Team");
}

#[sqlx::test(migrations = "./migrations")]
async fn get_team_returns_403_for_non_member(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (_, session2) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let token2 = session2.id.to_string();

    let (team, _) = common::create_test_team(&pool, "My Team", user1.id).await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, &format!("/teams/{}", team.id), &token2).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_members_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();
    let (team, _) = common::create_test_team(&pool, "My Team", user.id).await;

    let app = common::build_test_app(pool);
    let response =
        common::get_with_token(app, &format!("/teams/{}/members", team.id), &token).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let members = body.as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["user_id"], user.id.to_string());
    assert_eq!(members[0]["username"], "alissa");
    assert_eq!(members[0]["role"], "manager");
}

#[sqlx::test(migrations = "./migrations")]
async fn generate_invitation_code_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();
    let (team, _) = common::create_test_team(&pool, "My Team", user.id).await;

    let app = common::build_test_app(pool);
    let response =
        common::post_with_token(app, &format!("/teams/{}/invite", team.id), &token).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body["invitation_code"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn get_team_does_not_expose_invitation_code(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let token = session.id.to_string();
    let (team, _) = common::create_test_team(&pool, "My Team", user.id).await;

    let app = common::build_test_app(pool.clone());
    common::post_with_token(app, &format!("/teams/{}/invite", team.id), &token).await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, &format!("/teams/{}", team.id), &token).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body.get("invitation_code").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn join_team_returns_200_with_valid_code(pool: PgPool) {
    let (user1, session1) = common::create_test_user(&pool).await;
    let (_, session2) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let token1 = session1.id.to_string();
    let token2 = session2.id.to_string();

    let (team, _) = common::create_test_team(&pool, "My Team", user1.id).await;

    let app = common::build_test_app(pool.clone());
    let invite_response =
        common::post_with_token(app, &format!("/teams/{}/invite", team.id), &token1).await;
    let body = to_bytes(invite_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let code = body["invitation_code"].as_str().unwrap().to_string();

    let app = common::build_test_app(pool);
    let response =
        common::post_json_with_token(app, "/teams/join", &token2, json!({ "code": code })).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["member"]["role"], "observer");
}

#[sqlx::test(migrations = "./migrations")]
async fn transfer_manager_returns_204(pool: PgPool) {
    let (user1, session1) = common::create_test_user(&pool).await;
    let (user2, session2) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let token1 = session1.id.to_string();
    let token2 = session2.id.to_string();

    let (team, _) = common::create_test_team(&pool, "My Team", user1.id).await;

    let app = common::build_test_app(pool.clone());
    let invite_response =
        common::post_with_token(app, &format!("/teams/{}/invite", team.id), &token1).await;
    let body = to_bytes(invite_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let code = body["invitation_code"].as_str().unwrap().to_string();

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(app, "/teams/join", &token2, json!({ "code": code })).await;

    let app = common::build_test_app(pool.clone());
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/transfer", team.id),
        &token1,
        json!({ "user_id": user2.id }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
