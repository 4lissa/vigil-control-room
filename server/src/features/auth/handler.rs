use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use time::Duration;
use validator::Validate;

use crate::AppState;
use crate::features::auth::{
    dto::{
        AuthResponse, GithubCallbackQuery, LoginRequest, RegisterRequest, UpdateProfileRequest,
        UserResponse,
    },
    model::Session,
    service,
};
use crate::shared::{
    error::{AppError, validation_message},
    middleware::AuthUser,
};

const OAUTH_STATE_COOKIE: &str = "oauth_state";

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let (user, session) =
        service::register(&state.db, &req.username, &req.email, &req.password).await?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token: session.id.to_string(),
            user: user.into(),
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let (user, session) = service::login(&state.db, &req.email, &req.password).await?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            token: session.id.to_string(),
            user: user.into(),
        }),
    ))
}

pub async fn github_redirect(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let (url, oauth_state) = service::github_authorize_url(&state.config);

    let cookie = Cookie::build((OAUTH_STATE_COOKIE, oauth_state))
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(10))
        .path("/")
        .build();

    (jar.add(cookie), Redirect::to(&url))
}

pub async fn github_callback(
    State(state): State<AppState>,
    Query(query): Query<GithubCallbackQuery>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let expected_state = jar
        .get(OAUTH_STATE_COOKIE)
        .map(|cookie| cookie.value().to_string());
    let jar = jar.remove(OAUTH_STATE_COOKIE);

    let frontend_url = state
        .config
        .cors_origin
        .to_str()
        .expect("CORS_ORIGIN was validated as a valid header value at startup");

    let redirect_url = match complete_github_sign_in(&state, &query, expected_state).await {
        Ok(session) => format!("{frontend_url}/oauth/callback?token={}", session.id),
        Err(_) => format!("{frontend_url}/login?error=oauth_failed"),
    };

    (jar, Redirect::to(&redirect_url))
}

async fn complete_github_sign_in(
    state: &AppState,
    query: &GithubCallbackQuery,
    expected_state: Option<String>,
) -> Result<Session, AppError> {
    let code = query.code.as_deref().ok_or(AppError::Unauthorized)?;
    let received_state = query.state.as_deref().ok_or(AppError::Unauthorized)?;

    if expected_state.as_deref() != Some(received_state) {
        return Err(AppError::Unauthorized);
    }

    let (_user, session) =
        service::handle_github_callback(&state.db, &state.http_client, &state.config, code).await?;

    Ok(session)
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
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

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
