use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::incidents::model::{Incident, IncidentState, Severity, TimelineEntry};
use crate::features::incidents::repo;
use crate::features::teams::repo as teams_repo;
use crate::shared::error::AppError;

pub async fn create_incident(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
    title: &str,
    description: &str,
    severity: Severity,
) -> Result<Incident, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_create_incident() {
        return Err(AppError::Forbidden(
            "Only managers can create an incident".into(),
        ));
    }

    repo::insert_incident(
        pool,
        Uuid::now_v7(),
        team_id,
        title,
        description,
        &severity,
        requester_id,
    )
    .await
}

pub async fn get_incidents(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
) -> Result<Vec<Incident>, AppError> {
    require_member(pool, team_id, requester_id).await?;
    repo::find_incidents_by_team_id(pool, team_id).await
}

pub async fn get_incident(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    requester_id: Uuid,
) -> Result<Incident, AppError> {
    require_member(pool, team_id, requester_id).await?;
    require_incident(pool, team_id, incident_id).await
}

pub async fn acknowledge_incident(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    requester_id: Uuid,
) -> Result<Incident, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_acknowledge_incident() {
        return Err(AppError::Forbidden(
            "Only responders and managers can acknowledge an incident".into(),
        ));
    }

    let incident = require_incident(pool, team_id, incident_id).await?;
    let new_state = incident
        .acknowledge()
        .map_err(|e| AppError::Conflict(e.into()))?;

    repo::update_incident_state(pool, incident_id, &new_state, None).await
}

pub async fn escalate_incident(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    requester_id: Uuid,
    new_severity: Option<Severity>,
) -> Result<Incident, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_escalate_incident() {
        return Err(AppError::Forbidden(
            "Only responders and managers can escalate an incident".into(),
        ));
    }

    let incident = require_incident(pool, team_id, incident_id).await?;
    let new_state = incident
        .escalate()
        .map_err(|e| AppError::Conflict(e.into()))?;

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    let updated = repo::update_incident_state(&mut *tx, incident_id, &new_state, None).await?;

    let updated = if let Some(severity) = new_severity {
        repo::update_incident_severity(&mut *tx, incident_id, &severity).await?
    } else {
        updated
    };

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok(updated)
}

pub async fn resolve_incident(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    requester_id: Uuid,
) -> Result<Incident, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_close_incident() {
        return Err(AppError::Forbidden(
            "Only managers can resolve an incident".into(),
        ));
    }

    let incident = require_incident(pool, team_id, incident_id).await?;
    let new_state = incident
        .resolve()
        .map_err(|e| AppError::Conflict(e.into()))?;

    repo::update_incident_state(
        pool,
        incident_id,
        &new_state,
        Some(OffsetDateTime::now_utc()),
    )
    .await
}

pub async fn assign_responder(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    requester_id: Uuid,
    target_user_id: Option<Uuid>,
) -> Result<Incident, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_assign_responder() {
        return Err(AppError::Forbidden(
            "Only managers can assign a responder".into(),
        ));
    }

    let incident = require_incident(pool, team_id, incident_id).await?;

    if incident.state == IncidentState::Resolved {
        return Err(AppError::Conflict(
            "Cannot assign a responder to a resolved incident".into(),
        ));
    }

    if let Some(user_id) = target_user_id {
        teams_repo::find_member(pool, team_id, user_id)
            .await?
            .ok_or(AppError::NotFound(
                "Target user is not a member of this team".into(),
            ))?;
    }

    repo::update_incident_assigned_to(pool, incident_id, target_user_id).await
}

pub async fn add_timeline_entry(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    requester_id: Uuid,
    content: &str,
) -> Result<TimelineEntry, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_add_timeline_entry() {
        return Err(AppError::Forbidden(
            "Only responders and managers can add a timeline entry".into(),
        ));
    }

    require_incident(pool, team_id, incident_id).await?;

    repo::insert_timeline_entry(pool, Uuid::now_v7(), incident_id, requester_id, content).await
}

pub async fn get_timeline_entries(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    requester_id: Uuid,
) -> Result<Vec<TimelineEntry>, AppError> {
    require_member(pool, team_id, requester_id).await?;
    require_incident(pool, team_id, incident_id).await?;
    repo::find_timeline_entries_by_incident_id(pool, incident_id).await
}

pub async fn edit_timeline_entry(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
    entry_id: Uuid,
    requester_id: Uuid,
    new_content: &str,
) -> Result<TimelineEntry, AppError> {
    require_member(pool, team_id, requester_id).await?;
    require_incident(pool, team_id, incident_id).await?;

    let entry = repo::find_timeline_entry_by_id(pool, entry_id)
        .await?
        .ok_or(AppError::NotFound("Timeline entry not found".into()))?;

    if !entry.can_edit(requester_id) {
        return Err(AppError::Forbidden(
            "Only the author can edit their timeline entry".into(),
        ));
    }

    repo::update_timeline_entry_content(pool, entry_id, new_content).await
}

async fn require_member(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<crate::features::teams::model::TeamMember, AppError> {
    teams_repo::find_member(pool, team_id, user_id)
        .await?
        .ok_or(AppError::Forbidden(
            "You are not a member of this team".into(),
        ))
}

async fn require_incident(
    pool: &PgPool,
    team_id: Uuid,
    incident_id: Uuid,
) -> Result<Incident, AppError> {
    let incident = repo::find_incident_by_id(pool, incident_id)
        .await?
        .ok_or(AppError::NotFound("Incident not found".into()))?;

    if incident.team_id != team_id {
        return Err(AppError::NotFound("Incident not found".into()));
    }

    Ok(incident)
}
