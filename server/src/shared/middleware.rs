use axum::{
    RequestExt,
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use uuid::Uuid;

use crate::AppState;
use crate::features::auth::repo;
use crate::shared::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let TypedHeader(Authorization(bearer)) = req
        .extract_parts_with_state::<TypedHeader<Authorization<Bearer>>, AppState>(&state)
        .await
        .map_err(|_| AppError::Unauthorized)?;

    let session_id = Uuid::parse_str(bearer.token()).map_err(|_| AppError::Unauthorized)?;

    let session = repo::find_session_by_id(&state.db, session_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if session.is_expired() {
        return Err(AppError::Unauthorized);
    }

    req.extensions_mut().insert(AuthUser {
        user_id: session.user_id,
        session_id: session.id,
    });

    Ok(next.run(req).await)
}
