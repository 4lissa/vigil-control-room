use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Pong,
    IncidentStateChanged {
        incident_id: Uuid,
        new_state: String,
        by: String,
    },
    IncidentEscalated {
        incident_id: Uuid,
        new_severity: String,
        by: String,
    },
    IncidentAssigned {
        incident_id: Uuid,
        assigned_to: Option<Uuid>,
        by: String,
    },
    TimelineEntryAdded {
        incident_id: Uuid,
        entry_id: Uuid,
        author: String,
        content: String,
        created_at: i64,
    },
    TimelineEntryEdited {
        incident_id: Uuid,
        entry_id: Uuid,
        new_content: String,
        edited_at: i64,
    },
    PresenceUpdate {
        incident_id: Uuid,
        watchers: Vec<String>,
    },
    ReleaseStepValidated {
        release_id: Uuid,
        step: String,
        by: String,
    },
    ReleaseStateChanged {
        release_id: Uuid,
        new_state: String,
    },
    PrivateMessageReceived {
        from: String,
        to: String,
        content: String,
        at: i64,
    },
    ReactionAdded {
        incident_id: Uuid,
        entry_id: Uuid,
        emoji: String,
        by: String,
    },
    ReactionRemoved {
        incident_id: Uuid,
        entry_id: Uuid,
        emoji: String,
        by: String,
    },
    MemberJoined {
        team_id: Uuid,
        member: String,
        role: String,
    },
    MemberKicked {
        team_id: Uuid,
        member: String,
        by: String,
    },
    MemberBanned {
        team_id: Uuid,
        member: String,
        until: Option<i64>,
        by: String,
    },
    RuleTriggered {
        rule_name: String,
        result: String,
        incident_id: Option<Uuid>,
    },
    RuleFailed {
        rule_name: String,
        error: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Ping,
    Watch { incident_id: Uuid },
    Unwatch { incident_id: Uuid },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pong_serializes_correctly() {
        let event = WsEvent::Pong;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#"{"type":"pong"}"#);
    }

    #[test]
    fn incident_state_changed_serializes_correctly() {
        let event = WsEvent::IncidentStateChanged {
            incident_id: Uuid::nil(),
            new_state: "acknowledged".into(),
            by: "alissa".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"incident_state_changed""#));
        assert!(json.contains(r#""new_state":"acknowledged""#));
    }

    #[test]
    fn private_message_received_serializes_correctly() {
        let event = WsEvent::PrivateMessageReceived {
            from: "alissa".into(),
            to: "bob".into(),
            content: "hey".into(),
            at: 1718000000,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            r#"{"type":"private_message_received","from":"alissa","to":"bob","content":"hey","at":1718000000}"#
        );
    }

    #[test]
    fn member_joined_serializes_correctly() {
        let event = WsEvent::MemberJoined {
            team_id: Uuid::nil(),
            member: "alissa".into(),
            role: "observer".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"member_joined""#));
        assert!(json.contains(r#""role":"observer""#));
    }

    #[test]
    fn member_banned_serializes_correctly() {
        let event = WsEvent::MemberBanned {
            team_id: Uuid::nil(),
            member: "alissa".into(),
            until: None,
            by: "bob".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"member_banned""#));
        assert!(json.contains(r#""until":null"#));
    }

    #[test]
    fn reaction_added_serializes_correctly() {
        let event = WsEvent::ReactionAdded {
            incident_id: Uuid::nil(),
            entry_id: Uuid::nil(),
            emoji: "+1".into(),
            by: "alissa".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"reaction_added""#));
        assert!(json.contains(r#""emoji":"+1""#));
    }

    #[test]
    fn rule_triggered_serializes_correctly() {
        let event = WsEvent::RuleTriggered {
            rule_name: "CI failure -> Incident".into(),
            result: "incident_created".into(),
            incident_id: Some(Uuid::nil()),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""type":"rule_triggered""#));
        assert!(json.contains(r#""result":"incident_created""#));
    }

    #[test]
    fn rule_failed_serializes_correctly() {
        let event = WsEvent::RuleFailed {
            rule_name: "CI failure -> Incident".into(),
            error: "service_unavailable".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            r#"{"type":"rule_failed","rule_name":"CI failure -> Incident","error":"service_unavailable"}"#
        );
    }

    #[test]
    fn ping_deserializes_correctly() {
        let json = r#"{"type":"ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }

    #[test]
    fn watch_deserializes_correctly() {
        let id = Uuid::now_v7();
        let json = format!(r#"{{"type":"watch","incident_id":"{}"}}"#, id);
        let msg: ClientMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(msg, ClientMessage::Watch { .. }));
    }

    #[test]
    fn unknown_client_message_fails_gracefully() {
        let json = r#"{"type":"unknown"}"#;
        let result = serde_json::from_str::<ClientMessage>(json);
        assert!(result.is_err());
    }
}
