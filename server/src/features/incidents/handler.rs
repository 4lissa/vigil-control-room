use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

use crate::features::incidents::dto::{
    AddReactionRequest, AddTimelineEntryRequest, AssignResponderRequest, CreateIncidentRequest,
    EditTimelineEntryRequest, EmojiDto, EscalateIncidentRequest, IncidentResponse,
    ReactionResponse, ReactionSummaryResponse, TimelineEntryResponse,
};
use crate::features::incidents::model::ReactionEmoji;
use crate::features::incidents::service;
use crate::shared::error::{AppError, validation_message};
use crate::shared::middleware::AuthUser;
use crate::shared::ws::event::WsEvent;
use crate::state::AppState;

pub async fn create_incident(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
    Json(req): Json<CreateIncidentRequest>,
) -> Result<(StatusCode, Json<IncidentResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let description = req.description.unwrap_or_default();

    let (incident, blocked_release_state) = service::create_incident(
        &state.db,
        team_id,
        auth_user.user_id,
        &req.title,
        &description,
        req.severity.into(),
        req.release_id,
    )
    .await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::IncidentStateChanged {
                incident_id: incident.id,
                new_state: "open".into(),
                by: auth_user.username.clone(),
            },
        )
        .await;

    if let Some(release_state) = blocked_release_state {
        state
            .hub
            .broadcast_to_team(
                team_id,
                &WsEvent::ReleaseStateChanged {
                    release_id: incident.release_id.unwrap(),
                    new_state: release_state.as_str().into(),
                },
            )
            .await;
    }

    Ok((StatusCode::CREATED, Json(incident.into())))
}

pub async fn list_incidents(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<IncidentResponse>>, AppError> {
    let incidents = service::get_incidents(&state.db, team_id, auth_user.user_id).await?;
    Ok(Json(incidents.into_iter().map(Into::into).collect()))
}

pub async fn get_incident(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<IncidentResponse>, AppError> {
    let incident =
        service::get_incident(&state.db, team_id, incident_id, auth_user.user_id).await?;
    Ok(Json(incident.into()))
}

pub async fn acknowledge_incident(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<IncidentResponse>, AppError> {
    let incident =
        service::acknowledge_incident(&state.db, team_id, incident_id, auth_user.user_id).await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::IncidentStateChanged {
                incident_id,
                new_state: "acknowledged".into(),
                by: auth_user.username.clone(),
            },
        )
        .await;

    Ok(Json(incident.into()))
}

pub async fn escalate_incident(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<EscalateIncidentRequest>,
) -> Result<Json<IncidentResponse>, AppError> {
    let new_severity = req.severity.map(Into::into);
    let incident = service::escalate_incident(
        &state.db,
        team_id,
        incident_id,
        auth_user.user_id,
        new_severity,
    )
    .await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::IncidentEscalated {
                incident_id,
                new_severity: incident.severity.as_str().to_string(),
                by: auth_user.username.clone(),
            },
        )
        .await;

    Ok(Json(incident.into()))
}

pub async fn resolve_incident(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<IncidentResponse>, AppError> {
    let (incident, unblocked_release_state) =
        service::resolve_incident(&state.db, team_id, incident_id, auth_user.user_id).await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::IncidentStateChanged {
                incident_id,
                new_state: "resolved".into(),
                by: auth_user.username.clone(),
            },
        )
        .await;

    if let Some(release_state) = unblocked_release_state {
        state
            .hub
            .broadcast_to_team(
                team_id,
                &WsEvent::ReleaseStateChanged {
                    release_id: incident.release_id.unwrap(),
                    new_state: release_state.as_str().into(),
                },
            )
            .await;
    }

    Ok(Json(incident.into()))
}

pub async fn assign_responder(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AssignResponderRequest>,
) -> Result<Json<IncidentResponse>, AppError> {
    let incident = service::assign_responder(
        &state.db,
        team_id,
        incident_id,
        auth_user.user_id,
        req.user_id,
    )
    .await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::IncidentAssigned {
                incident_id,
                assigned_to: req.user_id,
                by: auth_user.username.clone(),
            },
        )
        .await;

    Ok(Json(incident.into()))
}

pub async fn add_timeline_entry(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AddTimelineEntryRequest>,
) -> Result<(StatusCode, Json<TimelineEntryResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let entry = service::add_timeline_entry(
        &state.db,
        team_id,
        incident_id,
        auth_user.user_id,
        &req.content,
    )
    .await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::TimelineEntryAdded {
                incident_id,
                entry_id: entry.id,
                author: auth_user.username.clone(),
                content: entry.content.clone(),
                created_at: entry.created_at.unix_timestamp(),
            },
        )
        .await;

    Ok((StatusCode::CREATED, Json(entry.into())))
}

pub async fn list_timeline_entries(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<TimelineEntryResponse>>, AppError> {
    let entries =
        service::get_timeline_entries(&state.db, team_id, incident_id, auth_user.user_id).await?;
    Ok(Json(entries.into_iter().map(Into::into).collect()))
}

pub async fn edit_timeline_entry(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id, entry_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(req): Json<EditTimelineEntryRequest>,
) -> Result<Json<TimelineEntryResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let entry = service::edit_timeline_entry(
        &state.db,
        team_id,
        incident_id,
        entry_id,
        auth_user.user_id,
        &req.content,
    )
    .await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::TimelineEntryEdited {
                incident_id,
                entry_id,
                new_content: entry.content.clone(),
                edited_at: entry.edited_at.unwrap().unix_timestamp(),
            },
        )
        .await;

    Ok(Json(entry.into()))
}

pub async fn list_available_emojis() -> Json<Vec<EmojiDto>> {
    Json(ReactionEmoji::all().into_iter().map(Into::into).collect())
}

pub async fn list_reactions(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<ReactionSummaryResponse>>, AppError> {
    let summaries =
        service::get_reactions(&state.db, team_id, incident_id, auth_user.user_id).await?;

    Ok(Json(summaries.into_iter().map(Into::into).collect()))
}

pub async fn add_reaction(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id, entry_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(req): Json<AddReactionRequest>,
) -> Result<(StatusCode, Json<ReactionResponse>), AppError> {
    let reaction = service::add_reaction(
        &state.db,
        team_id,
        incident_id,
        entry_id,
        auth_user.user_id,
        req.emoji.into(),
    )
    .await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::ReactionAdded {
                incident_id,
                entry_id,
                emoji: reaction.emoji.as_str().to_string(),
                by: auth_user.username.clone(),
            },
        )
        .await;

    Ok((StatusCode::CREATED, Json(reaction.into())))
}

pub async fn remove_reaction(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((team_id, incident_id, entry_id, emoji)): Path<(Uuid, Uuid, Uuid, EmojiDto)>,
) -> Result<StatusCode, AppError> {
    let emoji: ReactionEmoji = emoji.into();

    service::remove_reaction(
        &state.db,
        team_id,
        incident_id,
        entry_id,
        auth_user.user_id,
        emoji,
    )
    .await?;

    state
        .hub
        .broadcast_to_team(
            team_id,
            &WsEvent::ReactionRemoved {
                incident_id,
                entry_id,
                emoji: emoji.as_str().to_string(),
                by: auth_user.username.clone(),
            },
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}
