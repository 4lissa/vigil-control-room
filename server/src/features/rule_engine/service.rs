use sqlx::PgPool;
use uuid::Uuid;

use crate::features::auth::model::ServiceKind;
use crate::features::auth::repo as auth_repo;
use crate::features::incidents::model::Severity;
use crate::features::incidents::service as incidents_service;
use crate::features::rule_engine::model::{ReactionType, Rule, render_template};
use crate::features::rule_engine::repo;
use crate::features::rule_engine::webhooks::github as github_webhook;
use crate::features::teams::repo as teams_repo;
use crate::shared::config::Config;
use crate::shared::crypto;
use crate::shared::error::AppError;

const SUPPORTED_TRIGGERS: &[(&str, &str)] = &[("github", "workflow_run")];

#[derive(Debug, Clone)]
pub enum RuleExecution {
    Triggered {
        result: String,
        incident_id: Option<Uuid>,
    },
    Failed {
        error: String,
    },
}

#[allow(clippy::too_many_arguments)]
pub async fn create_rule(
    pool: &PgPool,
    config: &Config,
    team_id: Uuid,
    requester_id: Uuid,
    name: &str,
    enabled: bool,
    trigger_service: &str,
    trigger_event: &str,
    trigger_filters: serde_json::Value,
    webhook_secret: &str,
    reaction_type: &str,
    reaction_payload: serde_json::Value,
) -> Result<Rule, AppError> {
    require_manager(pool, team_id, requester_id).await?;

    if !SUPPORTED_TRIGGERS.contains(&(trigger_service, trigger_event)) {
        return Err(AppError::ValidationError(format!(
            "Unsupported trigger: {trigger_service}/{trigger_event}"
        )));
    }

    let reaction_type = ReactionType::parse(reaction_type).ok_or_else(|| {
        AppError::ValidationError(format!("Unsupported reaction type: {reaction_type}"))
    })?;

    let encrypted_webhook_secret = crypto::encrypt(&config.token_encryption_key, webhook_secret);

    repo::insert_rule(
        pool,
        Uuid::now_v7(),
        team_id,
        name,
        enabled,
        trigger_service,
        trigger_event,
        &trigger_filters,
        &encrypted_webhook_secret,
        reaction_type,
        &reaction_payload,
        requester_id,
    )
    .await
}

pub async fn get_rules(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
) -> Result<Vec<Rule>, AppError> {
    require_manager(pool, team_id, requester_id).await?;
    repo::find_rules_by_team_id(pool, team_id).await
}

pub async fn set_rule_enabled(
    pool: &PgPool,
    team_id: Uuid,
    rule_id: Uuid,
    requester_id: Uuid,
    enabled: bool,
) -> Result<Rule, AppError> {
    require_manager(pool, team_id, requester_id).await?;
    require_rule(pool, team_id, rule_id).await?;

    repo::update_rule_enabled(pool, rule_id, enabled).await
}

pub async fn delete_rule(
    pool: &PgPool,
    team_id: Uuid,
    rule_id: Uuid,
    requester_id: Uuid,
) -> Result<(), AppError> {
    require_manager(pool, team_id, requester_id).await?;
    require_rule(pool, team_id, rule_id).await?;

    repo::delete_rule(pool, rule_id).await
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_github_webhook(
    pool: &PgPool,
    http_client: &reqwest::Client,
    config: &Config,
    rule_id: Uuid,
    event_type: &str,
    signature: &str,
    raw_body: &[u8],
) -> Result<(Rule, Option<RuleExecution>), AppError> {
    let rule = repo::find_rule_by_id(pool, rule_id)
        .await?
        .ok_or(AppError::NotFound("Rule not found".into()))?;

    let secret = crypto::decrypt(&config.token_encryption_key, &rule.encrypted_webhook_secret)
        .ok_or(AppError::InternalError)?;

    if !github_webhook::verify_signature(secret.as_bytes(), raw_body, signature) {
        return Err(AppError::Unauthorized);
    }

    if !rule.enabled || rule.trigger_service != "github" || rule.trigger_event != event_type {
        return Ok((rule, None));
    }

    let payload: github_webhook::WorkflowRunPayload = serde_json::from_slice(raw_body)
        .map_err(|_| AppError::BadRequest("Invalid GitHub webhook payload".into()))?;

    let context = github_webhook::build_context(&payload);

    if !rule.matches(event_type, &context) {
        return Ok((rule, None));
    }

    let execution = execute_reaction(pool, http_client, config, &rule, &context).await;
    Ok((rule, Some(execution)))
}

async fn execute_reaction(
    pool: &PgPool,
    http_client: &reqwest::Client,
    config: &Config,
    rule: &Rule,
    context: &serde_json::Value,
) -> RuleExecution {
    let Some(created_by) = rule.created_by else {
        return RuleExecution::Failed {
            error: "This rule's creator account no longer exists".into(),
        };
    };

    match rule.reaction_type {
        ReactionType::VigilCreateIncident => {
            execute_vigil_create_incident(pool, rule, created_by, context).await
        }
        ReactionType::HttpPost => {
            execute_http_post(pool, http_client, config, rule, created_by, context).await
        }
    }
}

async fn execute_vigil_create_incident(
    pool: &PgPool,
    rule: &Rule,
    created_by: Uuid,
    context: &serde_json::Value,
) -> RuleExecution {
    let title_template = rule
        .reaction_payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let title = render_template(title_template, context);

    let body = rule
        .reaction_payload
        .get("body")
        .and_then(|v| v.as_str())
        .map(|template| render_template(template, context))
        .unwrap_or_default();

    let severity_str = rule
        .reaction_payload
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");

    let Some(severity) = Severity::parse(severity_str) else {
        return RuleExecution::Failed {
            error: format!("Invalid severity '{severity_str}' in rule reaction payload"),
        };
    };

    match incidents_service::create_incident(
        pool,
        rule.team_id,
        created_by,
        &title,
        &body,
        severity,
        None,
    )
    .await
    {
        Ok((incident, _)) => RuleExecution::Triggered {
            result: "incident_created".into(),
            incident_id: Some(incident.id),
        },
        Err(e) => RuleExecution::Failed {
            error: format!("Failed to create incident: {e:?}"),
        },
    }
}

async fn execute_http_post(
    pool: &PgPool,
    http_client: &reqwest::Client,
    config: &Config,
    rule: &Rule,
    created_by: Uuid,
    context: &serde_json::Value,
) -> RuleExecution {
    let Some(url) = rule.reaction_payload.get("url").and_then(|v| v.as_str()) else {
        return RuleExecution::Failed {
            error: "Missing 'url' in rule reaction payload".into(),
        };
    };

    let body = rule
        .reaction_payload
        .get("body")
        .and_then(|v| v.as_str())
        .map(|template| render_template(template, context))
        .unwrap_or_default();

    let token = match auth_repo::find_connected_service(pool, created_by, ServiceKind::Http).await {
        Ok(Some(connected)) => {
            crypto::decrypt(&config.token_encryption_key, &connected.encrypted_token)
        }
        Ok(None) => None,
        Err(e) => {
            return RuleExecution::Failed {
                error: format!("Failed to look up connected HTTP token: {e:?}"),
            };
        }
    };

    let mut request = http_client.post(url).body(body);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }

    match request.send().await {
        Ok(response) if response.status().is_success() => RuleExecution::Triggered {
            result: "http_post_sent".into(),
            incident_id: None,
        },
        Ok(response) => RuleExecution::Failed {
            error: format!("HTTP request failed with status {}", response.status()),
        },
        Err(e) => RuleExecution::Failed {
            error: format!("Failed to send HTTP request: {e}"),
        },
    }
}

async fn require_manager(pool: &PgPool, team_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let member = teams_repo::find_member(pool, team_id, user_id)
        .await?
        .ok_or(AppError::Forbidden(
            "You are not a member of this team".into(),
        ))?;

    if !member.role.can_configure_rules() {
        return Err(AppError::Forbidden(
            "Only managers can configure rules".into(),
        ));
    }

    Ok(())
}

async fn require_rule(pool: &PgPool, team_id: Uuid, rule_id: Uuid) -> Result<Rule, AppError> {
    let rule = repo::find_rule_by_id(pool, rule_id)
        .await?
        .ok_or(AppError::NotFound("Rule not found".into()))?;

    if rule.team_id != team_id {
        return Err(AppError::NotFound("Rule not found".into()));
    }

    Ok(rule)
}
