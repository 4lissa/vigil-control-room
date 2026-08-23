use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub github_id: Option<String>,
    pub language: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct SessionWithUsername {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceKind {
    Github,
    Http,
}

#[derive(Debug, Clone)]
pub struct ConnectedService {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service: ServiceKind,
    pub encrypted_token: Vec<u8>,
    pub created_at: OffsetDateTime,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        self.expires_at < OffsetDateTime::now_utc()
    }
}

impl SessionWithUsername {
    pub fn is_expired(&self) -> bool {
        self.expires_at < OffsetDateTime::now_utc()
    }
}

impl ServiceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceKind::Github => "github",
            ServiceKind::Http => "http",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "github" => Some(ServiceKind::Github),
            "http" => Some(ServiceKind::Http),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_expiring_at(expires_at: OffsetDateTime) -> Session {
        Session {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            created_at: OffsetDateTime::now_utc(),
            expires_at,
        }
    }

    #[test]
    fn is_expired_returns_false_for_future_date() {
        let session = session_expiring_at(OffsetDateTime::now_utc() + time::Duration::days(1));
        assert!(!session.is_expired());
    }

    #[test]
    fn is_expired_returns_true_for_past_date() {
        let session = session_expiring_at(OffsetDateTime::now_utc() - time::Duration::days(1));
        assert!(session.is_expired());
    }

    #[test]
    fn service_kind_parse_round_trips_through_as_str() {
        for service in [ServiceKind::Github, ServiceKind::Http] {
            assert_eq!(ServiceKind::parse(service.as_str()), Some(service));
        }
    }

    #[test]
    fn service_kind_parse_rejects_unknown_value() {
        assert_eq!(ServiceKind::parse("discord"), None);
    }
}
