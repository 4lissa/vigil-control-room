use axum::{Json, extract::State, http::StatusCode};
use validator::Validate;

use crate::AppState;
use crate::features::auth::{
    dto::{LoginRequest, LoginResponse, RegisterRequest, UpdateProfileRequest, UserResponse},
    service,
};
use crate::shared::{error::AppError, middleware::AuthUser};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let user = service::register(&state.db, &req.username, &req.email, &req.password).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let (user, session) = service::login(&state.db, &req.email, &req.password).await?;

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            token: session.id.to_string(),
            user: user.into(),
        }),
    ))
}

pub async fn logout(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<StatusCode, AppError> {
    service::logout(&state.db, auth_user.session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = service::me(&state.db, auth_user.user_id).await?;
    Ok((StatusCode::OK, Json(user.into())))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let user = service::update_profile(
        &state.db,
        auth_user.user_id,
        req.username.as_deref(),
        req.language.as_deref(),
        req.new_password.as_deref(),
        req.old_password.as_deref(),
    )
    .await?;

    Ok((StatusCode::OK, Json(user.into())))
}
