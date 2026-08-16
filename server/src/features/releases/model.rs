use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseState {
    Created,
    InProgress,
    Completed,
    Cancelled,
    Blocked,
}

impl ReleaseState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReleaseState::Created => "created",
            ReleaseState::InProgress => "in_progress",
            ReleaseState::Completed => "completed",
            ReleaseState::Cancelled => "cancelled",
            ReleaseState::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Release {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub state: ReleaseState,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct ReleaseStep {
    pub id: Uuid,
    pub release_id: Uuid,
    pub name: String,
    pub position: i32,
    pub validated_at: Option<OffsetDateTime>,
    pub validated_by: Option<Uuid>,
}

impl Release {
    pub fn cancel(&self) -> Result<ReleaseState, &'static str> {
        match self.state {
            ReleaseState::Created | ReleaseState::InProgress | ReleaseState::Blocked => {
                Ok(ReleaseState::Cancelled)
            }
            ReleaseState::Completed => Err("A completed release cannot be cancelled"),
            ReleaseState::Cancelled => Err("Release is already cancelled"),
        }
    }

    pub fn block(&self) -> Result<ReleaseState, &'static str> {
        match self.state {
            ReleaseState::InProgress => Ok(ReleaseState::Blocked),
            _ => Err("Only an in-progress release can be blocked"),
        }
    }

    pub fn unblock(&self) -> Result<ReleaseState, &'static str> {
        match self.state {
            ReleaseState::Blocked => Ok(ReleaseState::InProgress),
            _ => Err("Only a blocked release can be unblocked"),
        }
    }
}

impl ReleaseStep {
    pub fn is_validated(&self) -> bool {
        self.validated_at.is_some()
    }

    pub fn can_validate(&self, all_steps: &[ReleaseStep]) -> bool {
        if self.is_validated() {
            return false;
        }

        all_steps
            .iter()
            .filter(|step| !step.is_validated())
            .min_by_key(|step| step.position)
            .is_some_and(|next| next.id == self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_release(state: ReleaseState) -> Release {
        Release {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            name: "v1.0.0".into(),
            state,
            created_by: None,
            created_at: OffsetDateTime::now_utc(),
            completed_at: None,
        }
    }

    fn make_step(position: i32, validated: bool) -> ReleaseStep {
        ReleaseStep {
            id: Uuid::now_v7(),
            release_id: Uuid::now_v7(),
            name: format!("step-{position}"),
            position,
            validated_at: validated.then(OffsetDateTime::now_utc),
            validated_by: None,
        }
    }

    #[test]
    fn created_release_can_be_cancelled() {
        let release = make_release(ReleaseState::Created);
        assert_eq!(release.cancel(), Ok(ReleaseState::Cancelled));
    }

    #[test]
    fn in_progress_release_can_be_cancelled() {
        let release = make_release(ReleaseState::InProgress);
        assert_eq!(release.cancel(), Ok(ReleaseState::Cancelled));
    }

    #[test]
    fn blocked_release_can_be_cancelled() {
        let release = make_release(ReleaseState::Blocked);
        assert_eq!(release.cancel(), Ok(ReleaseState::Cancelled));
    }

    #[test]
    fn completed_release_cannot_be_cancelled() {
        let release = make_release(ReleaseState::Completed);
        assert!(release.cancel().is_err());
    }

    #[test]
    fn cancelled_release_cannot_be_cancelled_again() {
        let release = make_release(ReleaseState::Cancelled);
        assert!(release.cancel().is_err());
    }

    #[test]
    fn in_progress_release_can_be_blocked() {
        let release = make_release(ReleaseState::InProgress);
        assert_eq!(release.block(), Ok(ReleaseState::Blocked));
    }

    #[test]
    fn created_release_cannot_be_blocked() {
        let release = make_release(ReleaseState::Created);
        assert!(release.block().is_err());
    }

    #[test]
    fn blocked_release_can_be_unblocked() {
        let release = make_release(ReleaseState::Blocked);
        assert_eq!(release.unblock(), Ok(ReleaseState::InProgress));
    }

    #[test]
    fn in_progress_release_cannot_be_unblocked() {
        let release = make_release(ReleaseState::InProgress);
        assert!(release.unblock().is_err());
    }

    #[test]
    fn first_step_can_be_validated_when_none_validated() {
        let steps = vec![make_step(0, false), make_step(1, false)];
        assert!(steps[0].can_validate(&steps));
    }

    #[test]
    fn second_step_cannot_be_validated_before_first() {
        let steps = vec![make_step(0, false), make_step(1, false)];
        assert!(!steps[1].can_validate(&steps));
    }

    #[test]
    fn second_step_can_be_validated_once_first_is_validated() {
        let steps = vec![make_step(0, true), make_step(1, false)];
        assert!(steps[1].can_validate(&steps));
    }

    #[test]
    fn already_validated_step_cannot_be_validated_again() {
        let steps = vec![make_step(0, true), make_step(1, false)];
        assert!(!steps[0].can_validate(&steps));
    }
}
