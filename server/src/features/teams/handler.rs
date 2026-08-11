use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::features::teams::{
    dto::{
        CreateTeamRequest, CreateTeamResponse, InviteCodeResponse, JoinTeamRequest,
        TeamMemberResponse, TeamResponse, TransferManagerRequest,
    },
    service,
};
use crate::shared::{
    error::{AppError, validation_message},
    middleware::AuthUser,
};

pub async fn create_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<CreateTeamResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let (team, member) = service::create_team(&state.db, &req.name, auth_user.user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateTeamResponse {
            team: team.into(),
            member: member.into(),
        }),
    ))
}

pub async fn get_my_teams(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<TeamResponse>>), AppError> {
    let teams = service::get_my_teams(&state.db, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(teams.into_iter().map(Into::into).collect()),
    ))
}

pub async fn get_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<(StatusCode, Json<TeamResponse>), AppError> {
    let team = service::get_team(&state.db, team_id, auth_user.user_id).await?;

    Ok((StatusCode::OK, Json(team.into())))
}

pub async fn get_members(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<TeamMemberResponse>>), AppError> {
    let members = service::get_members(&state.db, team_id, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(members.into_iter().map(Into::into).collect()),
    ))
}

pub async fn generate_invitation_code(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<(StatusCode, Json<InviteCodeResponse>), AppError> {
    let invitation_code =
        service::generate_invitation_code_for_team(&state.db, team_id, auth_user.user_id).await?;

    Ok((StatusCode::OK, Json(InviteCodeResponse { invitation_code })))
}

pub async fn join_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<JoinTeamRequest>,
) -> Result<(StatusCode, Json<CreateTeamResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let (team, member) = service::join_team(&state.db, &req.code, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(CreateTeamResponse {
            team: team.into(),
            member: member.into(),
        }),
    ))
}

pub async fn transfer_manager(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<TransferManagerRequest>,
) -> Result<StatusCode, AppError> {
    service::transfer_manager(&state.db, team_id, auth_user.user_id, req.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
