use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::features::rule_engine::dto::{
    AboutClientDto, AboutResponse, AboutServerDto, CreateRuleRequest, RuleResponse,
    UpdateRuleRequest,
};
use crate::features::rule_engine::model;
use crate::features::rule_engine::service::{self, RuleExecution};
use crate::shared::error::{AppError, validation_message};
use crate::shared::middleware::AuthUser;
use crate::shared::ws::event::WsEvent;
use crate::state::AppState;

pub async fn create_rule(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<RuleResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let rule = service::create_rule(
        &state.db,
        &state.config,
        team_id,
        auth_user.user_id,
        &req.name,
        req.enabled,
        &req.trigger.service,
        &req.trigger.event,
        req.trigger.filters,
        &req.webhook_secret,
        &req.reaction.reaction_type,
        req.reaction.payload,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(rule.into())))
}

pub async fn list_rules(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<RuleResponse>>, AppError> {
    let rules = service::get_rules(&state.db, team_id, auth_user.user_id).await?;
    Ok(Json(rules.into_iter().map(Into::into).collect()))
}

pub async fn update_rule(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, rule_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateRuleRequest>,
) -> Result<Json<RuleResponse>, AppError> {
    let rule =
        service::set_rule_enabled(&state.db, team_id, rule_id, auth_user.user_id, req.enabled)
            .await?;

    Ok(Json(rule.into()))
}

pub async fn delete_rule(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, rule_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    service::delete_rule(&state.db, team_id, rule_id, auth_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn github_webhook(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("Missing X-GitHub-Event header".into()))?;

    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let (rule, execution) = service::handle_github_webhook(
        &state.db,
        &state.http_client,
        &state.config,
        rule_id,
        event_type,
        signature,
        &body,
    )
    .await?;

    match execution {
        Some(RuleExecution::Triggered {
            result,
            incident_id,
        }) => {
            state
                .hub
                .broadcast_to_team(
                    rule.team_id,
                    &WsEvent::RuleTriggered {
                        rule_name: rule.name,
                        result,
                        incident_id,
                    },
                )
                .await;
        }
        Some(RuleExecution::Failed { error }) => {
            state
                .hub
                .broadcast_to_team(
                    rule.team_id,
                    &WsEvent::RuleFailed {
                        rule_name: rule.name,
                        error,
                    },
                )
                .await;
        }
        None => {}
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn about_json(State(state): State<AppState>, headers: HeaderMap) -> Json<AboutResponse> {
    let host = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').next().unwrap_or(v).trim().to_string())
        .unwrap_or_else(|| "unknown".into());

    Json(AboutResponse {
        client: AboutClientDto { host },
        server: AboutServerDto {
            current_time: OffsetDateTime::now_utc().unix_timestamp(),
            services: model::service_catalog().iter().map(Into::into).collect(),
        },
        token: state.config.kickoff_token_hash.clone(),
    })
}
