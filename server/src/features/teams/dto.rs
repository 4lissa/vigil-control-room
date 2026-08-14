use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::features::teams::model::{Role, Team, TeamMember, TeamMemberWithUsername};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct JoinTeamRequest {
    #[validate(length(min = 1))]
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferManagerRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub name: String,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
}

impl From<Team> for TeamResponse {
    fn from(team: Team) -> Self {
        Self {
            id: team.id,
            name: team.name,
            created_by: team.created_by,
            created_at: team.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InviteCodeResponse {
    pub invitation_code: String,
}

#[derive(Debug, Serialize)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub role: String,
    pub joined_at: OffsetDateTime,
}

impl From<TeamMember> for TeamMemberResponse {
    fn from(member: TeamMember) -> Self {
        Self {
            id: member.id,
            team_id: member.team_id,
            user_id: member.user_id,
            username: None,
            role: role_to_string(&member.role),
            joined_at: member.joined_at,
        }
    }
}

impl From<TeamMemberWithUsername> for TeamMemberResponse {
    fn from(member: TeamMemberWithUsername) -> Self {
        Self {
            id: member.id,
            team_id: member.team_id,
            user_id: member.user_id,
            username: Some(member.username),
            role: role_to_string(&member.role),
            joined_at: member.joined_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TeamMembershipResponse {
    pub team: TeamResponse,
    pub member: TeamMemberResponse,
}

fn role_to_string(role: &Role) -> String {
    match role {
        Role::Observer => "observer".into(),
        Role::Responder => "responder".into(),
        Role::Manager => "manager".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_team_request_accepts_valid_name() {
        let req = CreateTeamRequest {
            name: "My Team".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_team_request_rejects_short_name() {
        let req = CreateTeamRequest { name: "A".into() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_team_request_rejects_empty_name() {
        let req = CreateTeamRequest { name: "".into() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn join_team_request_rejects_empty_code() {
        let req = JoinTeamRequest { code: "".into() };
        assert!(req.validate().is_err());
    }
}
