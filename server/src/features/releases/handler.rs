use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

use crate::features::releases::dto::{CreateReleaseRequest, ReleaseResponse, ReleaseStepResponse};
use crate::features::releases::service;
use crate::shared::error::{AppError, validation_message};
use crate::shared::middleware::AuthUser;
use crate::shared::ws::event::WsEvent;
use crate::state::AppState;

pub async fn create_release(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<CreateReleaseRequest>,
) -> Result<(StatusCode, Json<ReleaseResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let release =
        service::create_release(&state.db, team_id, auth_user.user_id, &req.name, &req.steps)
            .await?;

    Ok((StatusCode::CREATED, Json(release.into())))
}

pub async fn list_releases(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<ReleaseResponse>>, AppError> {
    let releases = service::get_releases(&state.db, team_id, auth_user.user_id).await?;
    Ok(Json(releases.into_iter().map(Into::into).collect()))
}

pub async fn get_release(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, release_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ReleaseResponse>, AppError> {
    let release = service::get_release(&state.db, team_id, release_id, auth_user.user_id).await?;
    Ok(Json(release.into()))
}

pub async fn list_release_steps(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, release_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<ReleaseStepResponse>>, AppError> {
    let steps =
        service::get_release_steps(&state.db, team_id, release_id, auth_user.user_id).await?;
    Ok(Json(steps.into_iter().map(Into::into).collect()))
}

pub async fn validate_step(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, release_id, step_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<ReleaseStepResponse>, AppError> {
    let (step, new_release_state) =
        service::validate_step(&state.db, team_id, release_id, step_id, auth_user.user_id).await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::ReleaseStepValidated {
                release_id,
                step: step.name.clone(),
                by: auth_user.username.clone(),
            },
        )
        .await;

    if let Some(new_state) = new_release_state {
        state
            .hub
            .broadcast_to_team(
                team_id,
                &WsEvent::ReleaseStateChanged {
                    release_id,
                    new_state: new_state.as_str().into(),
                },
            )
            .await;
    }

    Ok(Json(step.into()))
}

pub async fn cancel_release(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, release_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ReleaseResponse>, AppError> {
    let release =
        service::cancel_release(&state.db, team_id, release_id, auth_user.user_id).await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::ReleaseStateChanged {
                release_id,
                new_state: release.state.as_str().into(),
            },
        )
        .await;

    Ok(Json(release.into()))
}
