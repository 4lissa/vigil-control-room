use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::features::teams::model::{
    Role, Team, TeamBanWithUsername, TeamMember, TeamMemberWithUsername,
};

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

#[derive(Debug, Deserialize)]
pub struct BanMemberRequest {
    pub until: Option<i64>,
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

#[derive(Debug, Serialize)]
pub struct TeamBanResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub banned_by: Option<Uuid>,
    pub until: Option<i64>,
    pub created_at: i64,
}

impl From<TeamBanWithUsername> for TeamBanResponse {
    fn from(ban: TeamBanWithUsername) -> Self {
        Self {
            id: ban.id,
            team_id: ban.team_id,
            user_id: ban.user_id,
            username: ban.username,
            banned_by: ban.banned_by,
            until: ban.until.map(|t| t.unix_timestamp()),
            created_at: ban.created_at.unix_timestamp(),
        }
    }
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

    #[test]
    fn team_ban_response_from_ban_maps_fields() {
        let ban = TeamBanWithUsername {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            username: "bob".into(),
            banned_by: Some(Uuid::now_v7()),
            until: None,
            created_at: OffsetDateTime::now_utc(),
        };
        let id = ban.id;

        let response: TeamBanResponse = ban.into();

        assert_eq!(response.id, id);
        assert_eq!(response.username, "bob");
        assert!(response.until.is_none());
    }
}
