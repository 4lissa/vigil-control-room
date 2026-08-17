use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::messages::model::Message;

#[derive(Debug, Deserialize, Validate)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub content: String,
    pub created_at: i64,
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            sender_id: message.sender_id,
            recipient_id: message.recipient_id,
            content: message.content,
            created_at: message.created_at.unix_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    #[test]
    fn send_message_request_accepts_valid_content() {
        let req = SendMessageRequest {
            content: "Hey, got a minute?".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn send_message_request_rejects_empty_content() {
        let req = SendMessageRequest { content: "".into() };
        assert!(req.validate().is_err());
    }

    #[test]
    fn send_message_request_rejects_content_too_long() {
        let req = SendMessageRequest {
            content: "a".repeat(2001),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn message_response_from_message_maps_fields() {
        let message = Message {
            id: Uuid::now_v7(),
            sender_id: Uuid::now_v7(),
            recipient_id: Uuid::now_v7(),
            content: "Hey, got a minute?".into(),
            created_at: OffsetDateTime::now_utc(),
        };
        let id = message.id;

        let response: MessageResponse = message.into();

        assert_eq!(response.id, id);
        assert_eq!(response.content, "Hey, got a minute?");
    }
}
