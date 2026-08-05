use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    ValidationError(String),
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required".to_string(),
            ),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::ValidationError(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR", msg)
            }
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
            ),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

pub fn validation_message(errors: &ValidationErrors) -> String {
    errors
        .field_errors()
        .iter()
        .map(|(field, field_errors)| {
            let field_name = capitalize(field);
            let error = &field_errors[0];

            match error.code.as_ref() {
                "length" => match (error.params.get("min"), error.params.get("max")) {
                    (Some(min), Some(max)) => {
                        format!("{field_name} must be between {min} and {max} characters")
                    }
                    _ => format!("{field_name} has an invalid length"),
                },
                "email" => format!("{field_name} must be a valid email address"),
                _ => format!("{field_name} is invalid"),
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn capitalize(s: &str) -> String {
    let s = s.replace('_', " ");
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use validator::Validate;

    #[derive(Validate)]
    struct TestInput {
        #[validate(length(min = 8, max = 72))]
        new_password: String,
        #[validate(email)]
        email: String,
    }

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn bad_request_maps_to_400_with_correct_body() {
        let response = AppError::BadRequest("Bad input".into()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "BAD_REQUEST");
        assert_eq!(body["error"]["message"], "Bad input");
    }

    #[tokio::test]
    async fn unauthorized_maps_to_401_without_leaking_details() {
        let response = AppError::Unauthorized.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "UNAUTHORIZED");
        assert_eq!(body["error"]["message"], "Authentication required");
    }

    #[tokio::test]
    async fn forbidden_maps_to_403_with_correct_body() {
        let response = AppError::Forbidden("Only managers can kick members".into()).into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "FORBIDDEN");
        assert_eq!(body["error"]["message"], "Only managers can kick members");
    }

    #[tokio::test]
    async fn not_found_maps_to_404_with_correct_body() {
        let response = AppError::NotFound("User not found".into()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "NOT_FOUND");
        assert_eq!(body["error"]["message"], "User not found");
    }

    #[tokio::test]
    async fn conflict_maps_to_409_with_correct_body() {
        let response = AppError::Conflict("Email already taken".into()).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "CONFLICT");
        assert_eq!(body["error"]["message"], "Email already taken");
    }

    #[tokio::test]
    async fn validation_error_maps_to_422_with_correct_body() {
        let response = AppError::ValidationError("Invalid field".into()).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert_eq!(body["error"]["message"], "Invalid field");
    }

    #[tokio::test]
    async fn internal_error_maps_to_500_without_leaking_details() {
        let response = AppError::InternalError.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_json(response).await;
        assert_eq!(body["error"]["code"], "INTERNAL_ERROR");
        assert_eq!(body["error"]["message"], "An internal error occurred");
    }

    #[test]
    fn validation_message_reports_length_bounds_without_leaking_value() {
        let input = TestInput {
            new_password: "short".into(),
            email: "alissa@example.com".into(),
        };
        let errors = input.validate().unwrap_err();
        let message = validation_message(&errors);

        assert_eq!(message, "New password must be between 8 and 72 characters");
        assert!(!message.contains("short"));
    }

    #[test]
    fn validation_message_reports_invalid_email() {
        let input = TestInput {
            new_password: "password123".into(),
            email: "not-an-email".into(),
        };
        let errors = input.validate().unwrap_err();
        let message = validation_message(&errors);

        assert_eq!(message, "Email must be a valid email address");
        assert!(!message.contains("not-an-email"));
    }
}
