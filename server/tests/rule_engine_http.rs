mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::json;
use sha2::Sha256;
use sqlx::PgPool;

type HmacSha256 = Hmac<Sha256>;

fn sign(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let hex: String = mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("sha256={hex}")
}

fn create_rule_body() -> serde_json::Value {
    json!({
        "name": "CI failure > Incident",
        "enabled": true,
        "trigger": {
            "service": "github",
            "event": "workflow_run",
            "filters": { "workflow_run.conclusion": "failure" },
        },
        "webhook_secret": "shhh",
        "reaction": {
            "type": "vigil_create_incident",
            "payload": { "title": "CI broken on {{repository.full_name}}", "severity": "high" },
        },
    })
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn create_rule_returns_201_for_manager(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool);
    let response = common::post_json_with_token(
        app,
        &format!("/teams/{}/rules", team.id),
        &token,
        create_rule_body(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = json_body(response).await;
    assert_eq!(body["name"], "CI failure > Incident");
    assert_eq!(body["trigger"]["event"], "workflow_run");
    assert_eq!(body["reaction"]["type"], "vigil_create_incident");
    assert!(body.get("webhook_secret").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn create_rule_returns_403_for_observer(pool: PgPool) {
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
    let body = json_body(invite).await;
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
        &format!("/teams/{}/rules", team.id),
        &session2.id.to_string(),
        create_rule_body(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn list_rules_returns_200(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    common::post_json_with_token(
        app,
        &format!("/teams/{}/rules", team.id),
        &token,
        create_rule_body(),
    )
    .await;

    let app = common::build_test_app(pool);
    let response = common::get_with_token(app, &format!("/teams/{}/rules", team.id), &token).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn update_rule_toggles_enabled(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let created = common::post_json_with_token(
        app,
        &format!("/teams/{}/rules", team.id),
        &token,
        create_rule_body(),
    )
    .await;
    let created = json_body(created).await;
    let rule_id = created["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::patch_json_with_token(
        app,
        &format!("/teams/{}/rules/{}", team.id, rule_id),
        &token,
        json!({ "enabled": false }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["enabled"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_rule_returns_204(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let created = common::post_json_with_token(
        app,
        &format!("/teams/{}/rules", team.id),
        &token,
        create_rule_body(),
    )
    .await;
    let created = json_body(created).await;
    let rule_id = created["id"].as_str().unwrap();

    let app = common::build_test_app(pool);
    let response = common::delete_with_token(
        app,
        &format!("/teams/{}/rules/{}", team.id, rule_id),
        &token,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn github_webhook_triggers_the_rule_and_creates_an_incident(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let created = common::post_json_with_token(
        app,
        &format!("/teams/{}/rules", team.id),
        &token,
        create_rule_body(),
    )
    .await;
    let created = json_body(created).await;
    let rule_id = created["id"].as_str().unwrap();

    let body = serde_json::to_vec(&json!({
        "action": "completed",
        "workflow_run": {
            "name": "CI",
            "conclusion": "failure",
            "html_url": "https://github.com/my-org/my-repo/actions/runs/1",
        },
        "repository": { "name": "my-repo", "full_name": "my-org/my-repo" },
    }))
    .unwrap();
    let signature = sign("shhh", &body);

    let app = common::build_test_app(pool.clone());
    let response = common::post_bytes_with_headers(
        app,
        &format!("/webhooks/github/{rule_id}"),
        &[
            ("Content-Type", "application/json"),
            ("X-GitHub-Event", "workflow_run"),
            ("X-Hub-Signature-256", &signature),
        ],
        body,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let app = common::build_test_app(pool);
    let incidents =
        common::get_with_token(app, &format!("/teams/{}/incidents", team.id), &token).await;
    let incidents = json_body(incidents).await;
    assert_eq!(incidents.as_array().unwrap().len(), 1);
    assert_eq!(incidents[0]["title"], "CI broken on my-org/my-repo");
}

#[sqlx::test(migrations = "./migrations")]
async fn github_webhook_returns_204_for_a_ping_event(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let created = common::post_json_with_token(
        app,
        &format!("/teams/{}/rules", team.id),
        &token,
        create_rule_body(),
    )
    .await;
    let created = json_body(created).await;
    let rule_id = created["id"].as_str().unwrap();

    let body =
        serde_json::to_vec(&json!({ "zen": "Keep it logically awesome.", "hook_id": 1 })).unwrap();
    let signature = sign("shhh", &body);

    let app = common::build_test_app(pool);
    let response = common::post_bytes_with_headers(
        app,
        &format!("/webhooks/github/{rule_id}"),
        &[
            ("Content-Type", "application/json"),
            ("X-GitHub-Event", "ping"),
            ("X-Hub-Signature-256", &signature),
        ],
        body,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn github_webhook_returns_401_for_a_bad_signature(pool: PgPool) {
    let (user, session) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let token = session.id.to_string();

    let app = common::build_test_app(pool.clone());
    let created = common::post_json_with_token(
        app,
        &format!("/teams/{}/rules", team.id),
        &token,
        create_rule_body(),
    )
    .await;
    let created = json_body(created).await;
    let rule_id = created["id"].as_str().unwrap();

    let body = b"{}".to_vec();
    let signature = sign("wrong-secret", &body);

    let app = common::build_test_app(pool);
    let response = common::post_bytes_with_headers(
        app,
        &format!("/webhooks/github/{rule_id}"),
        &[
            ("Content-Type", "application/json"),
            ("X-GitHub-Event", "workflow_run"),
            ("X-Hub-Signature-256", &signature),
        ],
        body,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn about_json_returns_200_with_the_service_catalog_and_a_sha256_token(pool: PgPool) {
    let app = common::build_test_app(pool);
    let response = common::get(app, "/about.json").await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;

    let token = body["token"].as_str().unwrap();
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));

    let services = body["server"]["services"].as_array().unwrap();
    assert!(services.iter().any(|s| s["name"] == "github"));
    assert!(services.iter().any(|s| s["name"] == "vigil"));
}
