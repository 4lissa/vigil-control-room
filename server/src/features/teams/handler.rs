use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::features::auth::repo as auth_repo;
use crate::features::teams::{
    dto::{
        BanMemberRequest, CreateTeamRequest, InviteCodeResponse, JoinTeamRequest, TeamBanResponse,
        TeamMemberResponse, TeamMembershipResponse, TeamResponse, TransferManagerRequest,
    },
    service,
};
use crate::shared::{
    error::{AppError, validation_message},
    middleware::AuthUser,
    ws::event::WsEvent,
};

pub async fn create_team(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<TeamMembershipResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let (team, member) = service::create_team(&state.db, &req.name, auth_user.user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(TeamMembershipResponse {
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
) -> Result<(StatusCode, Json<TeamMembershipResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let (team, member) = service::join_team(&state.db, &req.code, auth_user.user_id).await?;

    let response = TeamMembershipResponse {
        team: team.into(),
        member: member.into(),
    };

    state
        .hub
        .broadcast_to_team(
            response.team.id,
            &WsEvent::MemberJoined {
                team_id: response.team.id,
                member: auth_user.username.clone(),
                role: response.member.role.clone(),
            },
        )
        .await;

    Ok((StatusCode::OK, Json(response)))
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

pub async fn kick_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let target = auth_repo::find_user_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::NotFound("Target user not found".into()))?;

    service::kick_member(&state.db, team_id, auth_user.user_id, user_id).await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::MemberKicked {
                team_id,
                member: target.username,
                by: auth_user.username.clone(),
            },
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn ban_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<BanMemberRequest>,
) -> Result<StatusCode, AppError> {
    let target = auth_repo::find_user_by_id(&state.db, user_id)
        .await?
        .ok_or(AppError::NotFound("Target user not found".into()))?;

    service::ban_member(&state.db, team_id, auth_user.user_id, user_id, req.until).await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::MemberBanned {
                team_id,
                member: target.username,
                until: req.until,
                by: auth_user.username.clone(),
            },
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unban_member(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    service::unban_member(&state.db, team_id, auth_user.user_id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_bans(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<TeamBanResponse>>, AppError> {
    let bans = service::get_active_bans(&state.db, team_id, auth_user.user_id).await?;

    Ok(Json(bans.into_iter().map(Into::into).collect()))
}
