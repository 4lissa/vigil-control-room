use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionType {
    VigilCreateIncident,
    HttpPost,
}

impl ReactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReactionType::VigilCreateIncident => "vigil_create_incident",
            ReactionType::HttpPost => "http_post",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "vigil_create_incident" => Some(ReactionType::VigilCreateIncident),
            "http_post" => Some(ReactionType::HttpPost),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub trigger_service: String,
    pub trigger_event: String,
    pub trigger_filters: serde_json::Value,
    pub encrypted_webhook_secret: Vec<u8>,
    pub reaction_type: ReactionType,
    pub reaction_payload: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
}

impl Rule {
    pub fn matches(&self, event_type: &str, context: &serde_json::Value) -> bool {
        self.enabled
            && self.trigger_event == event_type
            && matches_filters(context, &self.trigger_filters)
    }
}

pub fn matches_filters(context: &serde_json::Value, filters: &serde_json::Value) -> bool {
    let Some(filters) = filters.as_object() else {
        return true;
    };

    filters
        .iter()
        .all(|(path, expected)| get_path(context, path) == Some(expected))
}

pub fn render_template(template: &str, context: &serde_json::Value) -> String {
    let mut result = String::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);
        rest = &rest[start + 2..];

        match rest.find("}}") {
            Some(end) => {
                let path = rest[..end].trim();
                if let Some(value) = get_path(context, path) {
                    result.push_str(&display_value(value));
                }
                rest = &rest[end + 2..];
            }
            None => {
                result.push_str("{{");
                return result + rest;
            }
        }
    }

    result + rest
}

fn get_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    path.split('.').try_fold(value, |v, key| v.get(key))
}

fn display_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceCapability {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceCatalogEntry {
    pub name: &'static str,
    pub actions: &'static [ServiceCapability],
    pub reactions: &'static [ServiceCapability],
}

pub fn service_catalog() -> &'static [ServiceCatalogEntry] {
    &[
        ServiceCatalogEntry {
            name: "github",
            actions: &[ServiceCapability {
                name: "workflow_run_failed",
                description: "A CI workflow run completes with a failure conclusion",
            }],
            reactions: &[],
        },
        ServiceCatalogEntry {
            name: "vigil",
            actions: &[],
            reactions: &[ServiceCapability {
                name: "create_incident",
                description: "Create a VIGIL incident with configurable severity and title",
            }],
        },
        ServiceCatalogEntry {
            name: "http",
            actions: &[],
            reactions: &[ServiceCapability {
                name: "send_post",
                description: "Send a POST request to an external URL",
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_rule(enabled: bool, trigger_event: &str, filters: serde_json::Value) -> Rule {
        Rule {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            name: "CI failure".into(),
            enabled,
            trigger_service: "github".into(),
            trigger_event: trigger_event.into(),
            trigger_filters: filters,
            encrypted_webhook_secret: vec![],
            reaction_type: ReactionType::VigilCreateIncident,
            reaction_payload: json!({}),
            created_by: None,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn reaction_type_round_trips_through_str() {
        for reaction_type in [ReactionType::VigilCreateIncident, ReactionType::HttpPost] {
            assert_eq!(
                ReactionType::parse(reaction_type.as_str()),
                Some(reaction_type)
            );
        }
    }

    #[test]
    fn unknown_reaction_type_returns_none() {
        assert_eq!(ReactionType::parse("discord_send_message"), None);
    }

    #[test]
    fn disabled_rule_never_matches() {
        let rule = make_rule(false, "workflow_run", json!({}));
        assert!(!rule.matches("workflow_run", &json!({})));
    }

    #[test]
    fn rule_does_not_match_a_different_event() {
        let rule = make_rule(true, "workflow_run", json!({}));
        assert!(!rule.matches("pull_request", &json!({})));
    }

    #[test]
    fn rule_with_no_filters_matches_any_payload() {
        let rule = make_rule(true, "workflow_run", json!({}));
        assert!(rule.matches("workflow_run", &json!({ "anything": true })));
    }

    #[test]
    fn matches_filters_checks_every_key() {
        let context = json!({ "workflow_run": { "conclusion": "failure" }, "repository": { "full_name": "my-org/my-repo" } });
        let filters = json!({ "workflow_run.conclusion": "failure", "repository.full_name": "my-org/my-repo" });

        assert!(matches_filters(&context, &filters));
    }

    #[test]
    fn matches_filters_fails_if_one_key_differs() {
        let context = json!({ "workflow_run": { "conclusion": "success" } });
        let filters = json!({ "workflow_run.conclusion": "failure" });

        assert!(!matches_filters(&context, &filters));
    }

    #[test]
    fn matches_filters_fails_if_path_is_missing() {
        let context = json!({ "repository": { "full_name": "my-org/my-repo" } });
        let filters = json!({ "workflow_run.conclusion": "failure" });

        assert!(!matches_filters(&context, &filters));
    }

    #[test]
    fn render_template_substitutes_nested_paths() {
        let context = json!({ "repository": { "full_name": "my-org/my-repo" }, "workflow_run": { "html_url": "https://example.com/run/1" } });
        let rendered = render_template(
            "CI broken on {{repository.full_name}} — [View run]({{workflow_run.html_url}})",
            &context,
        );

        assert_eq!(
            rendered,
            "CI broken on my-org/my-repo — [View run](https://example.com/run/1)"
        );
    }

    #[test]
    fn render_template_leaves_unresolved_placeholders_empty() {
        let rendered = render_template("Hello {{unknown.path}}!", &json!({}));
        assert_eq!(rendered, "Hello !");
    }

    #[test]
    fn render_template_without_placeholders_is_unchanged() {
        let rendered = render_template("no placeholders here", &json!({}));
        assert_eq!(rendered, "no placeholders here");
    }

    #[test]
    fn render_template_handles_unterminated_placeholder() {
        let rendered = render_template("broken {{repository.full_name", &json!({}));
        assert_eq!(rendered, "broken {{repository.full_name");
    }

    #[test]
    fn service_catalog_lists_github_as_an_action_and_vigil_as_a_reaction() {
        let catalog = service_catalog();

        let github = catalog.iter().find(|s| s.name == "github").unwrap();
        assert!(!github.actions.is_empty());
        assert!(github.reactions.is_empty());

        let vigil = catalog.iter().find(|s| s.name == "vigil").unwrap();
        assert!(vigil.actions.is_empty());
        assert!(!vigil.reactions.is_empty());

        let http = catalog.iter().find(|s| s.name == "http").unwrap();
        assert!(http.actions.is_empty());
        assert!(!http.reactions.is_empty());
    }
}
