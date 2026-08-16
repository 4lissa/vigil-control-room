use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum IncidentState {
    Open,
    Acknowledged,
    Escalated,
    Resolved,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Incident {
    pub id: Uuid,
    pub team_id: Uuid,
    pub title: String,
    pub description: String,
    pub state: IncidentState,
    pub severity: Severity,
    pub created_by: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub resolved_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub id: Uuid,
    pub incident_id: Uuid,
    pub author_id: Option<Uuid>,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub edited_at: Option<OffsetDateTime>,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

impl Incident {
    pub fn acknowledge(&self) -> Result<IncidentState, &'static str> {
        match self.state {
            IncidentState::Open => Ok(IncidentState::Acknowledged),
            _ => Err("Only an open incident can be acknowledged"),
        }
    }

    pub fn escalate(&self) -> Result<IncidentState, &'static str> {
        match self.state {
            IncidentState::Acknowledged | IncidentState::Escalated => Ok(IncidentState::Escalated),
            _ => Err("Only an acknowledged or escalated incident can be escalated"),
        }
    }

    pub fn resolve(&self) -> Result<IncidentState, &'static str> {
        match self.state {
            IncidentState::Open | IncidentState::Acknowledged | IncidentState::Escalated => {
                Ok(IncidentState::Resolved)
            }
            IncidentState::Resolved => Err("Incident is already resolved"),
        }
    }
}

impl TimelineEntry {
    pub fn can_edit(&self, user_id: Uuid) -> bool {
        self.author_id == Some(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_incident(state: IncidentState) -> Incident {
        Incident {
            id: Uuid::now_v7(),
            team_id: Uuid::now_v7(),
            title: "Test incident".into(),
            description: "".into(),
            state,
            severity: Severity::High,
            created_by: None,
            assigned_to: None,
            created_at: OffsetDateTime::now_utc(),
            resolved_at: None,
        }
    }

    fn make_entry(author_id: Option<Uuid>) -> TimelineEntry {
        TimelineEntry {
            id: Uuid::now_v7(),
            incident_id: Uuid::now_v7(),
            author_id,
            content: "hello".into(),
            created_at: OffsetDateTime::now_utc(),
            edited_at: None,
        }
    }

    #[test]
    fn open_incident_can_be_acknowledged() {
        let incident = make_incident(IncidentState::Open);
        assert_eq!(incident.acknowledge(), Ok(IncidentState::Acknowledged));
    }

    #[test]
    fn acknowledged_incident_cannot_be_acknowledged_again() {
        let incident = make_incident(IncidentState::Acknowledged);
        assert!(incident.acknowledge().is_err());
    }

    #[test]
    fn acknowledged_incident_can_be_escalated() {
        let incident = make_incident(IncidentState::Acknowledged);
        assert_eq!(incident.escalate(), Ok(IncidentState::Escalated));
    }

    #[test]
    fn escalated_incident_can_be_escalated_again() {
        let incident = make_incident(IncidentState::Escalated);
        assert_eq!(incident.escalate(), Ok(IncidentState::Escalated));
    }

    #[test]
    fn open_incident_cannot_be_escalated() {
        let incident = make_incident(IncidentState::Open);
        assert!(incident.escalate().is_err());
    }

    #[test]
    fn resolved_incident_cannot_be_escalated() {
        let incident = make_incident(IncidentState::Resolved);
        assert!(incident.escalate().is_err());
    }

    #[test]
    fn open_incident_can_be_resolved() {
        let incident = make_incident(IncidentState::Open);
        assert_eq!(incident.resolve(), Ok(IncidentState::Resolved));
    }

    #[test]
    fn acknowledged_incident_can_be_resolved() {
        let incident = make_incident(IncidentState::Acknowledged);
        assert_eq!(incident.resolve(), Ok(IncidentState::Resolved));
    }

    #[test]
    fn escalated_incident_can_be_resolved() {
        let incident = make_incident(IncidentState::Escalated);
        assert_eq!(incident.resolve(), Ok(IncidentState::Resolved));
    }

    #[test]
    fn resolved_incident_cannot_be_resolved_again() {
        let incident = make_incident(IncidentState::Resolved);
        assert!(incident.resolve().is_err());
    }

    #[test]
    fn author_can_edit_their_entry() {
        let user_id = Uuid::now_v7();
        let entry = make_entry(Some(user_id));
        assert!(entry.can_edit(user_id));
    }

    #[test]
    fn non_author_cannot_edit_entry() {
        let entry = make_entry(Some(Uuid::now_v7()));
        assert!(!entry.can_edit(Uuid::now_v7()));
    }

    #[test]
    fn entry_with_deleted_author_cannot_be_edited() {
        let entry = make_entry(None);
        assert!(!entry.can_edit(Uuid::now_v7()));
    }
}
