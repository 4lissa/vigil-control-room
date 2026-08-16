use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::releases::model::{Release, ReleaseState, ReleaseStep};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateReleaseRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1))]
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseStateDto {
    Created,
    InProgress,
    Completed,
    Cancelled,
    Blocked,
}

impl From<ReleaseState> for ReleaseStateDto {
    fn from(state: ReleaseState) -> Self {
        match state {
            ReleaseState::Created => ReleaseStateDto::Created,
            ReleaseState::InProgress => ReleaseStateDto::InProgress,
            ReleaseState::Completed => ReleaseStateDto::Completed,
            ReleaseState::Cancelled => ReleaseStateDto::Cancelled,
            ReleaseState::Blocked => ReleaseStateDto::Blocked,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReleaseStepResponse {
    pub id: Uuid,
    pub release_id: Uuid,
    pub name: String,
    pub position: i32,
    pub validated_at: Option<i64>,
    pub validated_by: Option<Uuid>,
}

impl From<ReleaseStep> for ReleaseStepResponse {
    fn from(step: ReleaseStep) -> Self {
        Self {
            id: step.id,
            release_id: step.release_id,
            name: step.name,
            position: step.position,
            validated_at: step.validated_at.map(|t| t.unix_timestamp()),
            validated_by: step.validated_by,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReleaseResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub state: ReleaseStateDto,
    pub created_by: Option<Uuid>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

impl From<Release> for ReleaseResponse {
    fn from(release: Release) -> Self {
        Self {
            id: release.id,
            team_id: release.team_id,
            name: release.name,
            state: release.state.into(),
            created_by: release.created_by,
            created_at: release.created_at.unix_timestamp(),
            completed_at: release.completed_at.map(|t| t.unix_timestamp()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    #[test]
    fn create_release_request_accepts_valid_input() {
        let req = CreateReleaseRequest {
            name: "v1.0.0".into(),
            steps: vec!["build".into(), "staging".into()],
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_release_request_rejects_empty_name() {
        let req = CreateReleaseRequest {
            name: "".into(),
            steps: vec!["build".into()],
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_release_request_rejects_name_too_long() {
        let req = CreateReleaseRequest {
            name: "a".repeat(201),
            steps: vec!["build".into()],
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_release_request_rejects_empty_steps() {
        let req = CreateReleaseRequest {
            name: "v1.0.0".into(),
            steps: vec![],
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn release_response_from_release_maps_fields() {
        let release = Release {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            name: "v1.0.0".into(),
            state: ReleaseState::InProgress,
            created_by: None,
            created_at: OffsetDateTime::now_utc(),
            completed_at: None,
        };
        let id = release.id;

        let response: ReleaseResponse = release.into();

        assert_eq!(response.id, id);
        assert_eq!(response.name, "v1.0.0");
        assert!(response.completed_at.is_none());
    }

    #[test]
    fn release_step_response_from_step_maps_fields() {
        let step = ReleaseStep {
            id: Uuid::now_v7(),
            release_id: Uuid::now_v7(),
            name: "staging".into(),
            position: 1,
            validated_at: None,
            validated_by: None,
        };
        let id = step.id;

        let response: ReleaseStepResponse = step.into();

        assert_eq!(response.id, id);
        assert_eq!(response.name, "staging");
        assert!(response.validated_at.is_none());
    }
}
