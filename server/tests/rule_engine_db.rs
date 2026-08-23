mod common;

use hmac::{Hmac, KeyInit, Mac};
use serde_json::json;
use sha2::Sha256;
use sqlx::PgPool;
use vigil_server::features::rule_engine::service::{self, RuleExecution};
use vigil_server::shared::config::Config;
use vigil_server::shared::error::AppError;

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

#[allow(clippy::too_many_arguments)]
async fn create_test_rule(
    pool: &PgPool,
    config: &Config,
    team_id: uuid::Uuid,
    manager_id: uuid::Uuid,
    filters: serde_json::Value,
    webhook_secret: &str,
) -> vigil_server::features::rule_engine::model::Rule {
    service::create_rule(
        pool,
        config,
        team_id,
        manager_id,
        "CI failure > Incident",
        true,
        "github",
        "workflow_run",
        filters,
        webhook_secret,
        "vigil_create_incident",
        json!({ "title": "CI broken on {{repository.full_name}}", "severity": "high" }),
    )
    .await
    .expect("failed to create test rule")
}

fn http_client() -> reqwest::Client {
    reqwest::Client::new()
}

fn workflow_run_payload(conclusion: &str, full_name: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "action": "completed",
        "workflow_run": {
            "name": "CI",
            "conclusion": conclusion,
            "html_url": "https://github.com/my-org/my-repo/actions/runs/1",
        },
        "repository": {
            "name": "my-repo",
            "full_name": full_name,
        },
    }))
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn create_rule_succeeds_for_manager(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = create_test_rule(&pool, &config, team.id, user.id, json!({}), "shhh").await;

    assert_eq!(rule.name, "CI failure > Incident");
    assert!(rule.enabled);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_rule_fails_for_non_manager(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;
    let config = Config::from_env();

    let result = service::create_rule(
        &pool,
        &config,
        team.id,
        user2.id,
        "CI failure > Incident",
        true,
        "github",
        "workflow_run",
        json!({}),
        "shhh",
        "vigil_create_incident",
        json!({}),
    )
    .await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_rule_fails_for_unsupported_trigger(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let result = service::create_rule(
        &pool,
        &config,
        team.id,
        user.id,
        "Pipeline failure",
        true,
        "gitlab",
        "pipeline_failed",
        json!({}),
        "shhh",
        "vigil_create_incident",
        json!({}),
    )
    .await;

    assert!(matches!(result, Err(AppError::ValidationError(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_rule_fails_for_unsupported_reaction_type(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let result = service::create_rule(
        &pool,
        &config,
        team.id,
        user.id,
        "CI failure > Discord",
        true,
        "github",
        "workflow_run",
        json!({}),
        "shhh",
        "discord_send_message",
        json!({}),
    )
    .await;

    assert!(matches!(result, Err(AppError::ValidationError(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_rule_succeeds_for_http_reaction_without_a_connected_token(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let result = service::create_rule(
        &pool,
        &config,
        team.id,
        user.id,
        "CI failure > HTTP",
        true,
        "github",
        "workflow_run",
        json!({}),
        "shhh",
        "http_post",
        json!({ "url": "https://example.com" }),
    )
    .await;

    assert!(result.is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn get_rules_returns_created_rules(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    create_test_rule(&pool, &config, team.id, user.id, json!({}), "shhh").await;

    let rules = service::get_rules(&pool, team.id, user.id).await.unwrap();

    assert_eq!(rules.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn set_rule_enabled_toggles_state(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = create_test_rule(&pool, &config, team.id, user.id, json!({}), "shhh").await;

    let updated = service::set_rule_enabled(&pool, team.id, rule.id, user.id, false)
        .await
        .unwrap();

    assert!(!updated.enabled);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_rule_removes_it(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = create_test_rule(&pool, &config, team.id, user.id, json!({}), "shhh").await;

    service::delete_rule(&pool, team.id, rule.id, user.id)
        .await
        .unwrap();

    let rules = service::get_rules(&pool, team.id, user.id).await.unwrap();
    assert!(rules.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn handle_github_webhook_matches_and_creates_incident(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = create_test_rule(
        &pool,
        &config,
        team.id,
        user.id,
        json!({ "workflow_run.conclusion": "failure" }),
        "shhh",
    )
    .await;

    let body = workflow_run_payload("failure", "my-org/my-repo");
    let signature = sign("shhh", &body);

    let (_, execution) = service::handle_github_webhook(
        &pool,
        &http_client(),
        &config,
        rule.id,
        "workflow_run",
        &signature,
        &body,
    )
    .await
    .unwrap();

    match execution {
        Some(RuleExecution::Triggered {
            result,
            incident_id,
        }) => {
            assert_eq!(result, "incident_created");
            assert!(incident_id.is_some());
        }
        other => panic!("expected a Triggered execution, got {other:?}"),
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn handle_github_webhook_returns_none_when_filters_dont_match(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = create_test_rule(
        &pool,
        &config,
        team.id,
        user.id,
        json!({ "workflow_run.conclusion": "failure" }),
        "shhh",
    )
    .await;

    let body = workflow_run_payload("success", "my-org/my-repo");
    let signature = sign("shhh", &body);

    let (_, execution) = service::handle_github_webhook(
        &pool,
        &http_client(),
        &config,
        rule.id,
        "workflow_run",
        &signature,
        &body,
    )
    .await
    .unwrap();

    assert!(execution.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn handle_github_webhook_ignores_events_the_rule_does_not_listen_for(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = create_test_rule(&pool, &config, team.id, user.id, json!({}), "shhh").await;

    let body =
        serde_json::to_vec(&json!({ "zen": "Keep it logically awesome.", "hook_id": 1 })).unwrap();
    let signature = sign("shhh", &body);

    let (_, execution) = service::handle_github_webhook(
        &pool,
        &http_client(),
        &config,
        rule.id,
        "ping",
        &signature,
        &body,
    )
    .await
    .unwrap();

    assert!(execution.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn handle_github_webhook_fails_with_a_bad_signature(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = create_test_rule(&pool, &config, team.id, user.id, json!({}), "shhh").await;

    let body = workflow_run_payload("failure", "my-org/my-repo");
    let signature = sign("wrong-secret", &body);

    let result = service::handle_github_webhook(
        &pool,
        &http_client(),
        &config,
        rule.id,
        "workflow_run",
        &signature,
        &body,
    )
    .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[sqlx::test(migrations = "./migrations")]
async fn handle_github_webhook_reports_a_failed_execution_for_an_invalid_severity(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = service::create_rule(
        &pool,
        &config,
        team.id,
        user.id,
        "CI failure > Incident",
        true,
        "github",
        "workflow_run",
        json!({}),
        "shhh",
        "vigil_create_incident",
        json!({ "title": "CI broken", "severity": "not-a-real-severity" }),
    )
    .await
    .unwrap();

    let body = workflow_run_payload("failure", "my-org/my-repo");
    let signature = sign("shhh", &body);

    let (_, execution) = service::handle_github_webhook(
        &pool,
        &http_client(),
        &config,
        rule.id,
        "workflow_run",
        &signature,
        &body,
    )
    .await
    .unwrap();

    assert!(matches!(execution, Some(RuleExecution::Failed { .. })));
}

#[sqlx::test(migrations = "./migrations")]
async fn handle_github_webhook_http_reaction_fails_when_url_is_missing(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;
    let config = Config::from_env();

    let rule = service::create_rule(
        &pool,
        &config,
        team.id,
        user.id,
        "CI failure > HTTP",
        true,
        "github",
        "workflow_run",
        json!({}),
        "shhh",
        "http_post",
        json!({}),
    )
    .await
    .unwrap();

    let body = workflow_run_payload("failure", "my-org/my-repo");
    let signature = sign("shhh", &body);

    let (_, execution) = service::handle_github_webhook(
        &pool,
        &http_client(),
        &config,
        rule.id,
        "workflow_run",
        &signature,
        &body,
    )
    .await
    .unwrap();

    assert!(matches!(execution, Some(RuleExecution::Failed { .. })));
}
