use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Observer,
    Responder,
    Manager,
}

#[derive(Debug, Clone)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub invitation_code: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
    pub joined_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct TeamMemberWithUsername {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub role: Role,
    pub joined_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct TeamBan {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub banned_by: Option<Uuid>,
    pub until: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl TeamBan {
    pub fn is_active(&self) -> bool {
        match self.until {
            None => true,
            Some(until) => until > OffsetDateTime::now_utc(),
        }
    }
}

impl Role {
    pub fn can_generate_invitation_code(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_update_member_role(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_transfer_manager(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_moderate_members(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_create_incident(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_acknowledge_incident(&self) -> bool {
        matches!(self, Role::Responder | Role::Manager)
    }

    pub fn can_escalate_incident(&self) -> bool {
        matches!(self, Role::Responder | Role::Manager)
    }

    pub fn can_assign_responder(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_close_incident(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_add_timeline_entry(&self) -> bool {
        matches!(self, Role::Responder | Role::Manager)
    }

    pub fn can_create_release(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_validate_release_step(&self) -> bool {
        matches!(self, Role::Responder | Role::Manager)
    }

    pub fn can_cancel_release(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_configure_rules(&self) -> bool {
        matches!(self, Role::Manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ban(until: Option<OffsetDateTime>) -> TeamBan {
        TeamBan {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            banned_by: None,
            until,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn permanent_ban_is_always_active() {
        let ban = make_ban(None);
        assert!(ban.is_active());
    }

    #[test]
    fn temporary_ban_is_active_before_expiry() {
        let ban = make_ban(Some(OffsetDateTime::now_utc() + time::Duration::days(1)));
        assert!(ban.is_active());
    }

    #[test]
    fn temporary_ban_is_inactive_after_expiry() {
        let ban = make_ban(Some(OffsetDateTime::now_utc() - time::Duration::days(1)));
        assert!(!ban.is_active());
    }

    #[test]
    fn observer_has_no_write_permissions() {
        let role = Role::Observer;
        assert!(!role.can_generate_invitation_code());
        assert!(!role.can_update_member_role());
        assert!(!role.can_transfer_manager());
        assert!(!role.can_moderate_members());
        assert!(!role.can_create_incident());
        assert!(!role.can_acknowledge_incident());
        assert!(!role.can_escalate_incident());
        assert!(!role.can_assign_responder());
        assert!(!role.can_close_incident());
        assert!(!role.can_add_timeline_entry());
        assert!(!role.can_create_release());
        assert!(!role.can_validate_release_step());
        assert!(!role.can_cancel_release());
        assert!(!role.can_configure_rules());
    }

    #[test]
    fn responder_can_act_but_not_manage() {
        let role = Role::Responder;
        assert!(!role.can_generate_invitation_code());
        assert!(!role.can_update_member_role());
        assert!(!role.can_transfer_manager());
        assert!(!role.can_moderate_members());
        assert!(!role.can_create_incident());
        assert!(role.can_acknowledge_incident());
        assert!(role.can_escalate_incident());
        assert!(!role.can_assign_responder());
        assert!(!role.can_close_incident());
        assert!(role.can_add_timeline_entry());
        assert!(!role.can_create_release());
        assert!(role.can_validate_release_step());
        assert!(!role.can_cancel_release());
        assert!(!role.can_configure_rules());
    }

    #[test]
    fn manager_has_all_permissions() {
        let role = Role::Manager;
        assert!(role.can_generate_invitation_code());
        assert!(role.can_update_member_role());
        assert!(role.can_transfer_manager());
        assert!(role.can_moderate_members());
        assert!(role.can_create_incident());
        assert!(role.can_acknowledge_incident());
        assert!(role.can_escalate_incident());
        assert!(role.can_assign_responder());
        assert!(role.can_close_incident());
        assert!(role.can_add_timeline_entry());
        assert!(role.can_create_release());
        assert!(role.can_validate_release_step());
        assert!(role.can_cancel_release());
        assert!(role.can_configure_rules());
    }
}
