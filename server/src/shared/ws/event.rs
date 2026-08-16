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
