use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::AppState;
use crate::features::auth::repo;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id:    Uuid,
    pub session_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let session_id = Uuid::parse_str(bearer.token())
            .map_err(|_| AppError::Unauthorized)?;

        let session = repo::find_session_by_id(&state.db, session_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if session.is_expired() {
            return Err(AppError::Unauthorized);
        }

        Ok(AuthUser {
            user_id:    session.user_id,
            session_id: session.id,
        })
    }
}
