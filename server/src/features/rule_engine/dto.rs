use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::rule_engine::model::{Rule, ServiceCapability, ServiceCatalogEntry};

#[derive(Debug, Deserialize)]
pub struct TriggerDto {
    pub service: String,
    pub event: String,
    #[serde(default = "default_filters")]
    pub filters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ReactionDto {
    #[serde(rename = "type")]
    pub reaction_type: String,
    #[serde(default = "default_filters")]
    pub payload: serde_json::Value,
}

fn default_filters() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRuleRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub enabled: bool,
    pub trigger: TriggerDto,
    #[validate(length(min = 1, max = 500))]
    pub webhook_secret: String,
    pub reaction: ReactionDto,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct TriggerResponseDto {
    pub service: String,
    pub event: String,
    pub filters: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ReactionResponseDto {
    #[serde(rename = "type")]
    pub reaction_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RuleResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub trigger: TriggerResponseDto,
    pub reaction: ReactionResponseDto,
    pub created_by: Option<Uuid>,
    pub created_at: i64,
}

impl From<Rule> for RuleResponse {
    fn from(rule: Rule) -> Self {
        Self {
            id: rule.id,
            team_id: rule.team_id,
            name: rule.name,
            enabled: rule.enabled,
            trigger: TriggerResponseDto {
                service: rule.trigger_service,
                event: rule.trigger_event,
                filters: rule.trigger_filters,
            },
            reaction: ReactionResponseDto {
                reaction_type: rule.reaction_type.as_str().to_string(),
                payload: rule.reaction_payload,
            },
            created_by: rule.created_by,
            created_at: rule.created_at.unix_timestamp(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AboutResponse {
    pub client: AboutClientDto,
    pub server: AboutServerDto,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AboutClientDto {
    pub host: String,
}

#[derive(Debug, Serialize)]
pub struct AboutServerDto {
    pub current_time: i64,
    pub services: Vec<AboutServiceDto>,
}

#[derive(Debug, Serialize)]
pub struct AboutServiceDto {
    pub name: String,
    pub actions: Vec<AboutCapabilityDto>,
    pub reactions: Vec<AboutCapabilityDto>,
}

#[derive(Debug, Serialize)]
pub struct AboutCapabilityDto {
    pub name: String,
    pub description: String,
}

impl From<&ServiceCapability> for AboutCapabilityDto {
    fn from(capability: &ServiceCapability) -> Self {
        Self {
            name: capability.name.to_string(),
            description: capability.description.to_string(),
        }
    }
}

impl From<&ServiceCatalogEntry> for AboutServiceDto {
    fn from(entry: &ServiceCatalogEntry) -> Self {
        Self {
            name: entry.name.to_string(),
            actions: entry.actions.iter().map(Into::into).collect(),
            reactions: entry.reactions.iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    use crate::features::rule_engine::model::ReactionType;

    #[test]
    fn create_rule_request_accepts_valid_input() {
        let req = CreateRuleRequest {
            name: "CI failure > Critical Incident".into(),
            enabled: true,
            trigger: TriggerDto {
                service: "github".into(),
                event: "workflow_run".into(),
                filters: serde_json::json!({ "workflow_run.conclusion": "failure" }),
            },
            webhook_secret: "shhh".into(),
            reaction: ReactionDto {
                reaction_type: "vigil_create_incident".into(),
                payload: serde_json::json!({ "title": "CI broken", "severity": "high" }),
            },
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_rule_request_rejects_empty_name() {
        let req = CreateRuleRequest {
            name: "".into(),
            enabled: true,
            trigger: TriggerDto {
                service: "github".into(),
                event: "workflow_run".into(),
                filters: default_filters(),
            },
            webhook_secret: "shhh".into(),
            reaction: ReactionDto {
                reaction_type: "vigil_create_incident".into(),
                payload: default_filters(),
            },
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_rule_request_rejects_empty_webhook_secret() {
        let req = CreateRuleRequest {
            name: "CI failure".into(),
            enabled: true,
            trigger: TriggerDto {
                service: "github".into(),
                event: "workflow_run".into(),
                filters: default_filters(),
            },
            webhook_secret: "".into(),
            reaction: ReactionDto {
                reaction_type: "vigil_create_incident".into(),
                payload: default_filters(),
            },
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn rule_response_from_rule_maps_fields() {
        let rule = Rule {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            name: "CI failure".into(),
            enabled: true,
            trigger_service: "github".into(),
            trigger_event: "workflow_run".into(),
            trigger_filters: serde_json::json!({ "workflow_run.conclusion": "failure" }),
            encrypted_webhook_secret: vec![1, 2, 3],
            reaction_type: ReactionType::VigilCreateIncident,
            reaction_payload: serde_json::json!({ "title": "CI broken" }),
            created_by: None,
            created_at: OffsetDateTime::now_utc(),
        };

        let response: RuleResponse = rule.into();

        assert_eq!(response.trigger.event, "workflow_run");
        assert_eq!(response.reaction.reaction_type, "vigil_create_incident");
    }
}
