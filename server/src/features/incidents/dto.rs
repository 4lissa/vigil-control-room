use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::incidents::model::{
    Incident, IncidentState, Reaction, ReactionEmoji, ReactionSummary, Severity, TimelineEntry,
};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateIncidentRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    pub severity: SeverityDto,
    pub release_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct EscalateIncidentRequest {
    pub severity: Option<SeverityDto>,
}

#[derive(Debug, Deserialize)]
pub struct AssignResponderRequest {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddTimelineEntryRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EditTimelineEntryRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct AddReactionRequest {
    pub emoji: EmojiDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SeverityDto {
    Low,
    Medium,
    High,
    Critical,
}

impl From<SeverityDto> for Severity {
    fn from(dto: SeverityDto) -> Self {
        match dto {
            SeverityDto::Low => Severity::Low,
            SeverityDto::Medium => Severity::Medium,
            SeverityDto::High => Severity::High,
            SeverityDto::Critical => Severity::Critical,
        }
    }
}

impl From<Severity> for SeverityDto {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Low => SeverityDto::Low,
            Severity::Medium => SeverityDto::Medium,
            Severity::High => SeverityDto::High,
            Severity::Critical => SeverityDto::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStateDto {
    Open,
    Acknowledged,
    Escalated,
    Resolved,
}

impl From<IncidentState> for IncidentStateDto {
    fn from(state: IncidentState) -> Self {
        match state {
            IncidentState::Open => IncidentStateDto::Open,
            IncidentState::Acknowledged => IncidentStateDto::Acknowledged,
            IncidentState::Escalated => IncidentStateDto::Escalated,
            IncidentState::Resolved => IncidentStateDto::Resolved,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EmojiDto {
    #[serde(rename = "+1")]
    ThumbsUp,
    #[serde(rename = "-1")]
    ThumbsDown,
    #[serde(rename = "eyes")]
    Eyes,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "check")]
    Check,
    #[serde(rename = "fire")]
    Fire,
}

impl From<EmojiDto> for ReactionEmoji {
    fn from(dto: EmojiDto) -> Self {
        match dto {
            EmojiDto::ThumbsUp => ReactionEmoji::ThumbsUp,
            EmojiDto::ThumbsDown => ReactionEmoji::ThumbsDown,
            EmojiDto::Eyes => ReactionEmoji::Eyes,
            EmojiDto::Warning => ReactionEmoji::Warning,
            EmojiDto::Check => ReactionEmoji::Check,
            EmojiDto::Fire => ReactionEmoji::Fire,
        }
    }
}

impl From<ReactionEmoji> for EmojiDto {
    fn from(emoji: ReactionEmoji) -> Self {
        match emoji {
            ReactionEmoji::ThumbsUp => EmojiDto::ThumbsUp,
            ReactionEmoji::ThumbsDown => EmojiDto::ThumbsDown,
            ReactionEmoji::Eyes => EmojiDto::Eyes,
            ReactionEmoji::Warning => EmojiDto::Warning,
            ReactionEmoji::Check => EmojiDto::Check,
            ReactionEmoji::Fire => EmojiDto::Fire,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IncidentResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub title: String,
    pub description: String,
    pub state: IncidentStateDto,
    pub severity: SeverityDto,
    pub created_by: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub release_id: Option<Uuid>,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

impl From<Incident> for IncidentResponse {
    fn from(incident: Incident) -> Self {
        Self {
            id: incident.id,
            team_id: incident.team_id,
            title: incident.title,
            description: incident.description,
            state: incident.state.into(),
            severity: incident.severity.into(),
            created_by: incident.created_by,
            assigned_to: incident.assigned_to,
            release_id: incident.release_id,
            created_at: incident.created_at.unix_timestamp(),
            resolved_at: incident.resolved_at.map(|t| t.unix_timestamp()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TimelineEntryResponse {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub author_id: Option<Uuid>,
    pub content: String,
    pub created_at: i64,
    pub edited_at: Option<i64>,
}

impl From<TimelineEntry> for TimelineEntryResponse {
    fn from(entry: TimelineEntry) -> Self {
        Self {
            id: entry.id,
            incident_id: entry.incident_id,
            author_id: entry.author_id,
            content: entry.content,
            created_at: entry.created_at.unix_timestamp(),
            edited_at: entry.edited_at.map(|t| t.unix_timestamp()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReactionResponse {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub user_id: Uuid,
    pub emoji: EmojiDto,
    pub created_at: i64,
}

impl From<Reaction> for ReactionResponse {
    fn from(reaction: Reaction) -> Self {
        Self {
            id: reaction.id,
            entry_id: reaction.entry_id,
            user_id: reaction.user_id,
            emoji: reaction.emoji.into(),
            created_at: reaction.created_at.unix_timestamp(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReactionSummaryResponse {
    pub entry_id: Uuid,
    pub emoji: EmojiDto,
    pub usernames: Vec<String>,
    pub reacted_by_me: bool,
}

impl From<ReactionSummary> for ReactionSummaryResponse {
    fn from(summary: ReactionSummary) -> Self {
        Self {
            entry_id: summary.entry_id,
            emoji: summary.emoji.into(),
            usernames: summary.usernames,
            reacted_by_me: summary.reacted_by_me,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    #[test]
    fn create_incident_request_accepts_valid_input() {
        let req = CreateIncidentRequest {
            title: "DB down".into(),
            description: Some("Investigating".into()),
            severity: SeverityDto::High,
            release_id: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_incident_request_rejects_empty_title() {
        let req = CreateIncidentRequest {
            title: "".into(),
            description: None,
            severity: SeverityDto::High,
            release_id: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_incident_request_rejects_title_too_long() {
        let req = CreateIncidentRequest {
            title: "a".repeat(201),
            description: None,
            severity: SeverityDto::High,
            release_id: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_incident_request_rejects_description_too_long() {
        let req = CreateIncidentRequest {
            title: "DB down".into(),
            description: Some("a".repeat(2001)),
            severity: SeverityDto::High,
            release_id: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn add_timeline_entry_request_rejects_empty_content() {
        let req = AddTimelineEntryRequest { content: "".into() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn edit_timeline_entry_request_rejects_empty_content() {
        let req = EditTimelineEntryRequest { content: "".into() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn severity_dto_round_trips_through_severity() {
        let severity: Severity = SeverityDto::Critical.into();
        assert_eq!(severity, Severity::Critical);

        let dto: SeverityDto = severity.into();
        assert_eq!(dto, SeverityDto::Critical);
    }

    #[test]
    fn incident_response_from_incident_maps_fields() {
        let incident = Incident {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            title: "DB down".into(),
            description: "".into(),
            state: IncidentState::Open,
            severity: Severity::High,
            created_by: None,
            assigned_to: None,
            release_id: None,
            created_at: OffsetDateTime::now_utc(),
            resolved_at: None,
        };
        let id = incident.id;

        let response: IncidentResponse = incident.into();

        assert_eq!(response.id, id);
        assert_eq!(response.title, "DB down");
        assert_eq!(response.severity, SeverityDto::High);
        assert!(response.resolved_at.is_none());
    }

    #[test]
    fn timeline_entry_response_from_entry_maps_fields() {
        let entry = TimelineEntry {
            id: Uuid::now_v7(),
            incident_id: Uuid::now_v7(),
            author_id: Some(Uuid::now_v7()),
            content: "Investigating".into(),
            created_at: OffsetDateTime::now_utc(),
            edited_at: None,
        };
        let id = entry.id;

        let response: TimelineEntryResponse = entry.into();

        assert_eq!(response.id, id);
        assert_eq!(response.content, "Investigating");
        assert!(response.edited_at.is_none());
    }

    #[test]
    fn emoji_dto_round_trips_through_reaction_emoji() {
        let emoji: ReactionEmoji = EmojiDto::ThumbsUp.into();
        assert_eq!(emoji, ReactionEmoji::ThumbsUp);

        let dto: EmojiDto = emoji.into();
        assert_eq!(dto, EmojiDto::ThumbsUp);
    }

    #[test]
    fn emoji_dto_serializes_to_the_short_code() {
        let json = serde_json::to_string(&EmojiDto::ThumbsUp).unwrap();
        assert_eq!(json, r#""+1""#);
    }

    #[test]
    fn reaction_response_from_reaction_maps_fields() {
        let reaction = Reaction {
            id: Uuid::now_v7(),
            entry_id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            emoji: ReactionEmoji::Fire,
            created_at: OffsetDateTime::now_utc(),
        };
        let id = reaction.id;

        let response: ReactionResponse = reaction.into();

        assert_eq!(response.id, id);
        assert_eq!(response.emoji, EmojiDto::Fire);
    }

    #[test]
    fn reaction_summary_response_from_summary_maps_fields() {
        let summary = ReactionSummary {
            entry_id: Uuid::now_v7(),
            emoji: ReactionEmoji::Eyes,
            usernames: vec!["alissa".into(), "bob".into()],
            reacted_by_me: true,
        };

        let response: ReactionSummaryResponse = summary.into();

        assert_eq!(response.emoji, EmojiDto::Eyes);
        assert_eq!(response.usernames, vec!["alissa", "bob"]);
        assert!(response.reacted_by_me);
    }
}
