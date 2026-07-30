use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::auth::model::User;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 72))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 72))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(length(min = 2, max = 10))]
    pub language: Option<String>,
    #[validate(length(min = 8, max = 72))]
    pub new_password: Option<String>,
    pub old_password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub language: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            language: user.language,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_request_accepts_valid_input() {
        let req = RegisterRequest {
            username: "alissa".into(),
            email: "alissa@example.com".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn register_request_rejects_short_username() {
        let req = RegisterRequest {
            username: "ab".into(),
            email: "alissa@example.com".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_request_rejects_invalid_email() {
        let req = RegisterRequest {
            username: "alissa".into(),
            email: "not-an-email".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn register_request_rejects_short_password() {
        let req = RegisterRequest {
            username: "alissa".into(),
            email: "alissa@example.com".into(),
            password: "short".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_request_accepts_valid_input() {
        let req = LoginRequest {
            email: "alissa@example.com".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn login_request_rejects_invalid_email() {
        let req = LoginRequest {
            email: "not-an-email".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_profile_request_accepts_all_fields_absent() {
        let req = UpdateProfileRequest {
            username: None,
            language: None,
            new_password: None,
            old_password: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn update_profile_request_rejects_short_language() {
        let req = UpdateProfileRequest {
            username: None,
            language: Some("x".into()),
            new_password: None,
            old_password: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn update_profile_request_rejects_short_new_password() {
        let req = UpdateProfileRequest {
            username: None,
            language: None,
            new_password: Some("short".into()),
            old_password: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn user_response_from_user_maps_fields() {
        let user = User {
            id: Uuid::now_v7(),
            username: "alissa".into(),
            email: "alissa@example.com".into(),
            password_hash: Some("hash".into()),
            github_id: None,
            language: "fr".into(),
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
        };
        let id = user.id;
        let response: UserResponse = user.into();

        assert_eq!(response.id, id);
        assert_eq!(response.username, "alissa");
        assert_eq!(response.email, "alissa@example.com");
        assert_eq!(response.language, "fr");
    }
}
