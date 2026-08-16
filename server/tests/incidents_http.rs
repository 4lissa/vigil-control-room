mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn create_incident_returns_201_for_manager(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["title"], "DB down");
    assert_eq!(body["state"], "open");
    assert_eq!(body["severity"], "high");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_incident_returns_403_for_observer(pool: PgPool) {
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
        &format!("/teams/{}/incidents", team.id),
        &session2.id.to_string(),
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_incidents_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;

    let app = common::build_test_app(pool);
    let response =
        common::get_with_token(app, &format!("/teams/{}/incidents", team.id), &token).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn acknowledge_incident_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let incident_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::post_with_token(
        app,
        &format!("/teams/{}/incidents/{}/acknowledge", team.id, incident_id),
        &token,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["state"], "acknowledged");
}

#[sqlx::test(migrations = "./migrations")]
async fn escalate_incident_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let incident_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool.clone());
    common::post_with_token(
        app,
        &format!("/teams/{}/incidents/{}/acknowledge", team.id, incident_id),
        &token,
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents/{}/escalate", team.id, incident_id),
        &token,
        json!({ "severity": "critical" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["state"], "escalated");
    assert_eq!(body["severity"], "critical");
}

#[sqlx::test(migrations = "./migrations")]
async fn resolve_incident_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let incident_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::post_with_token(
        app,
        &format!("/teams/{}/incidents/{}/resolve", team.id, incident_id),
        &token,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["state"], "resolved");
    assert!(body["resolved_at"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn add_timeline_entry_returns_201(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let incident_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents/{}/timeline", team.id, incident_id),
        &token,
        json!({ "content": "Investigating the issue" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["content"], "Investigating the issue");
}

#[sqlx::test(migrations = "./migrations")]
async fn edit_timeline_entry_returns_200_for_author(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &token,
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let incident_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents/{}/timeline", team.id, incident_id),
        &token,
        json!({ "content": "Initial note" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let entry_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::patch_json_with_token(
        app,
        &format!(
            "/teams/{}/incidents/{}/timeline/{}",
            team.id, incident_id, entry_id
        ),
        &token,
        json!({ "content": "Updated note" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["content"], "Updated note");
    assert!(body["edited_at"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn edit_timeline_entry_returns_403_for_non_author(pool: PgPool) {
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

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents", team.id),
        &session1.id.to_string(),
        json!({ "title": "DB down", "severity": "high" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let incident_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool.clone());
    let res = common::post_json_with_token(
        app,
        &format!("/teams/{}/incidents/{}/timeline", team.id, incident_id),
        &session1.id.to_string(),
        json!({ "content": "Initial note" }),
    )
    .await;
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let entry_id = body["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::patch_json_with_token(
        app,
        &format!(
            "/teams/{}/incidents/{}/timeline/{}",
            team.id, incident_id, entry_id
        ),
        &session2.id.to_string(),
        json!({ "content": "Hacked" }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
