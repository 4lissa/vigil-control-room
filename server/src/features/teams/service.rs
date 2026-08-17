use rand::Rng;
use rand::distributions::Alphanumeric;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::features::auth::repo as auth_repo;
use crate::features::teams::{
    model::{Role, Team, TeamBanWithUsername, TeamMember, TeamMemberWithUsername},
    repo,
};
use crate::shared::error::AppError;

fn generate_invitation_code() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

pub async fn create_team(
    pool: &PgPool,
    name: &str,
    creator_id: Uuid,
) -> Result<(Team, TeamMember), AppError> {
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    let team = repo::insert_team(&mut *tx, Uuid::now_v7(), name, creator_id).await?;

    let member = repo::insert_team_member(
        &mut *tx,
        Uuid::now_v7(),
        team.id,
        creator_id,
        &Role::Manager,
    )
    .await?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok((team, member))
}

pub async fn join_team(
    pool: &PgPool,
    invitation_code: &str,
    user_id: Uuid,
) -> Result<(Team, TeamMember), AppError> {
    let team = repo::find_team_by_invitation_code(pool, invitation_code)
        .await?
        .ok_or(AppError::NotFound("Invalid invitation code".into()))?;

    if repo::find_member(pool, team.id, user_id).await?.is_some() {
        return Err(AppError::Conflict("Already a member of this team".into()));
    }

    if repo::find_active_ban(pool, team.id, user_id)
        .await?
        .is_some()
    {
        return Err(AppError::Forbidden("You are banned from this team".into()));
    }

    let member =
        repo::insert_team_member(pool, Uuid::now_v7(), team.id, user_id, &Role::Observer).await?;

    Ok((team, member))
}

pub async fn get_my_teams(pool: &PgPool, user_id: Uuid) -> Result<Vec<Team>, AppError> {
    repo::find_teams_by_user_id(pool, user_id).await
}

pub async fn get_team(pool: &PgPool, team_id: Uuid, requester_id: Uuid) -> Result<Team, AppError> {
    require_member(pool, team_id, requester_id).await?;

    repo::find_team_by_id(pool, team_id)
        .await?
        .ok_or(AppError::NotFound("Team not found".into()))
}

pub async fn get_members(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
) -> Result<Vec<TeamMemberWithUsername>, AppError> {
    require_member(pool, team_id, requester_id).await?;
    repo::find_members_by_team_id(pool, team_id).await
}

pub async fn generate_invitation_code_for_team(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
) -> Result<String, AppError> {
    let actor = require_member(pool, team_id, requester_id).await?;

    if !actor.role.can_generate_invitation_code() {
        return Err(AppError::Forbidden(
            "Only managers can generate an invitation code".into(),
        ));
    }

    repo::update_invitation_code(pool, team_id, &generate_invitation_code())
        .await?
        .invitation_code
        .ok_or(AppError::InternalError)
}

pub async fn transfer_manager(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    let actor = require_member(pool, team_id, requester_id).await?;

    if !actor.role.can_transfer_manager() {
        return Err(AppError::Forbidden(
            "Only managers can transfer the manager role".into(),
        ));
    }

    if requester_id == target_user_id {
        return Err(AppError::BadRequest(
            "Cannot transfer manager role to yourself".into(),
        ));
    }

    let target = repo::find_member(pool, team_id, target_user_id)
        .await?
        .ok_or(AppError::NotFound(
            "Target user is not a member of this team".into(),
        ))?;

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    repo::update_member_role(&mut *tx, team_id, requester_id, &Role::Responder).await?;
    repo::update_member_role(&mut *tx, team_id, target.user_id, &Role::Manager).await?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok(())
}

pub async fn kick_member(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    let actor = require_member(pool, team_id, requester_id).await?;

    if !actor.role.can_moderate_members() {
        return Err(AppError::Forbidden(
            "Only managers can kick a member".into(),
        ));
    }

    if requester_id == target_user_id {
        return Err(AppError::BadRequest("Cannot kick yourself".into()));
    }

    repo::find_member(pool, team_id, target_user_id)
        .await?
        .ok_or(AppError::NotFound(
            "Target user is not a member of this team".into(),
        ))?;

    repo::delete_member(pool, team_id, target_user_id).await
}

pub async fn ban_member(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
    target_user_id: Uuid,
    until: Option<i64>,
) -> Result<(), AppError> {
    let actor = require_member(pool, team_id, requester_id).await?;

    if !actor.role.can_moderate_members() {
        return Err(AppError::Forbidden("Only managers can ban a member".into()));
    }

    if requester_id == target_user_id {
        return Err(AppError::BadRequest("Cannot ban yourself".into()));
    }

    let until = until
        .map(OffsetDateTime::from_unix_timestamp)
        .transpose()
        .map_err(|_| AppError::BadRequest("Invalid ban expiration".into()))?;

    if let Some(until) = until
        && until <= OffsetDateTime::now_utc()
    {
        return Err(AppError::BadRequest(
            "Ban expiration must be in the future".into(),
        ));
    }

    require_user(pool, target_user_id).await?;

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to begin transaction");
        AppError::InternalError
    })?;

    repo::insert_ban(
        &mut *tx,
        Uuid::now_v7(),
        team_id,
        target_user_id,
        requester_id,
        until,
    )
    .await?;

    repo::delete_member(&mut *tx, team_id, target_user_id).await?;

    tx.commit().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to commit transaction");
        AppError::InternalError
    })?;

    Ok(())
}

pub async fn unban_member(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    let actor = require_member(pool, team_id, requester_id).await?;

    if !actor.role.can_moderate_members() {
        return Err(AppError::Forbidden("Only managers can lift a ban".into()));
    }

    repo::delete_ban(pool, team_id, target_user_id).await
}

pub async fn get_active_bans(
    pool: &PgPool,
    team_id: Uuid,
    requester_id: Uuid,
) -> Result<Vec<TeamBanWithUsername>, AppError> {
    let actor = require_member(pool, team_id, requester_id).await?;

    if !actor.role.can_moderate_members() {
        return Err(AppError::Forbidden(
            "Only managers can view banned members".into(),
        ));
    }

    repo::find_active_bans_by_team_id(pool, team_id).await
}

async fn require_member(
    pool: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<TeamMember, AppError> {
    repo::find_member(pool, team_id, user_id)
        .await?
        .ok_or(AppError::Forbidden(
            "You are not a member of this team".into(),
        ))
}

async fn require_user(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    auth_repo::find_user_by_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound("Target user not found".into()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_invitation_code_has_correct_length() {
        let code = generate_invitation_code();
        assert_eq!(code.len(), 16);
    }

    #[test]
    fn generate_invitation_code_produces_distinct_values() {
        let code1 = generate_invitation_code();
        let code2 = generate_invitation_code();
        assert_ne!(code1, code2);
    }
}
