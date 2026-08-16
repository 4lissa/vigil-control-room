use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::incidents::repo as incidents_repo;
use crate::features::releases::model::{Release, ReleaseState, ReleaseStep};
use crate::features::releases::repo;
use crate::features::teams::repo as teams_repo;
use crate::shared::error::AppError;

pub async fn create_release(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
    name: &str,
    step_names: &[String],
) -> Result<Release, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_create_release() {
        return Err(AppError::Forbidden(
            "Only managers can create a release".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    let release =
        repo::insert_release(&mut *tx, Uuid::now_v7(), team_id, name, requester_id).await?;

    for (position, step_name) in step_names.iter().enumerate() {
        repo::insert_release_step(
            &mut *tx,
            Uuid::now_v7(),
            release.id,
            step_name,
            position as i32,
        )
        .await?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok(release)
}

pub async fn get_releases(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
) -> Result<Vec<Release>, AppError> {
    require_member(pool, team_id, requester_id).await?;
    repo::find_releases_by_team_id(pool, team_id).await
}

pub async fn get_release(
    pool: &PgPool,
    team_id: Uuid,
    release_id: Uuid,
    requester_id: Uuid,
) -> Result<Release, AppError> {
    require_member(pool, team_id, requester_id).await?;
    require_release(pool, team_id, release_id).await
}

pub async fn get_release_steps(
    pool: &PgPool,
    team_id: Uuid,
    release_id: Uuid,
    requester_id: Uuid,
) -> Result<Vec<ReleaseStep>, AppError> {
    require_member(pool, team_id, requester_id).await?;
    require_release(pool, team_id, release_id).await?;
    repo::find_steps_by_release_id(pool, release_id).await
}

pub async fn validate_step(
    pool: &PgPool,
    team_id: Uuid,
    release_id: Uuid,
    step_id: Uuid,
    requester_id: Uuid,
) -> Result<(ReleaseStep, Option<ReleaseState>), AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_validate_release_step() {
        return Err(AppError::Forbidden(
            "Only responders and managers can validate a release step".into(),
        ));
    }

    let release = require_release(pool, team_id, release_id).await?;

    if release.state != ReleaseState::Created && release.state != ReleaseState::InProgress {
        return Err(AppError::Conflict(
            "Only a created or in-progress release can have its steps validated".into(),
        ));
    }

    if incidents_repo::has_unresolved_incidents_for_release(pool, release_id).await? {
        return Err(AppError::Conflict(
            "This release has an unresolved incident linked to it".into(),
        ));
    }

    let steps = repo::find_steps_by_release_id(pool, release_id).await?;
    let step = steps
        .iter()
        .find(|s| s.id == step_id)
        .cloned()
        .ok_or(AppError::NotFound("Release step not found".into()))?;

    if !step.can_validate(&steps) {
        return Err(AppError::Conflict(
            "This step cannot be validated yet".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    let updated_step = repo::update_step_validated(&mut *tx, step_id, requester_id).await?;

    let all_validated = steps.iter().all(|s| s.id == step_id || s.is_validated());

    let new_state = if all_validated {
        Some(ReleaseState::Completed)
    } else if release.state == ReleaseState::Created {
        Some(ReleaseState::InProgress)
    } else {
        None
    };

    if let Some(new_state) = &new_state {
        let completed_at = (*new_state == ReleaseState::Completed).then(OffsetDateTime::now_utc);
        repo::update_release_state(&mut *tx, release_id, new_state, completed_at).await?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok((updated_step, new_state))
}

pub async fn cancel_release(
    pool: &PgPool,
    team_id: Uuid,
    release_id: Uuid,
    requester_id: Uuid,
) -> Result<Release, AppError> {
    let member = require_member(pool, team_id, requester_id).await?;

    if !member.role.can_cancel_release() {
        return Err(AppError::Forbidden(
            "Only managers can cancel a release".into(),
        ));
    }

    let release = require_release(pool, team_id, release_id).await?;
    let new_state = release.cancel().map_err(|e| AppError::Conflict(e.into()))?;

    repo::update_release_state(pool, release_id, &new_state, None).await
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

async fn require_release(
    pool: &PgPool,
    team_id: Uuid,
    release_id: Uuid,
) -> Result<Release, AppError> {
    let release = repo::find_release_by_id(pool, release_id)
        .await?
        .ok_or(AppError::NotFound("Release not found".into()))?;

    if release.team_id != team_id {
        return Err(AppError::NotFound("Release not found".into()));
    }

    Ok(release)
}
