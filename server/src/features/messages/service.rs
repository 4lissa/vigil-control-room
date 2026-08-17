use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::features::messages::model::Message;
use crate::features::messages::repo;
use crate::features::teams::repo as teams_repo;
use crate::shared::error::AppError;

pub async fn send_message(
    pool: &PgPool,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: &str,
) -> Result<Message, AppError> {
    if sender_id == recipient_id {
        return Err(AppError::BadRequest(
            "Cannot send a message to yourself".into(),
        ));
    }

    require_shared_team(pool, sender_id, recipient_id).await?;

    repo::insert_message(pool, Uuid::now_v7(), sender_id, recipient_id, content).await
}

pub async fn get_conversation(
    pool: &PgPool,
    requester_id: Uuid,
    other_user_id: Uuid,
) -> Result<Vec<Message>, AppError> {
    require_shared_team(pool, requester_id, other_user_id).await?;
    repo::find_conversation(pool, requester_id, other_user_id).await
}

pub async fn list_conversations(
    pool: &PgPool,
    requester_id: Uuid,
) -> Result<Vec<Message>, AppError> {
    let conversations = repo::find_latest_message_per_conversation(pool, requester_id).await?;

    let mut visible = Vec::with_capacity(conversations.len());
    for message in conversations {
        let counterpart_id = if message.sender_id == requester_id {
            message.recipient_id
        } else {
            message.sender_id
        };

        if shares_team(pool, requester_id, counterpart_id).await? {
            visible.push(message);
        }
    }

    Ok(visible)
}

async fn shares_team(pool: &PgPool, user_a: Uuid, user_b: Uuid) -> Result<bool, AppError> {
    let teams_a = teams_repo::find_teams_by_user_id(pool, user_a).await?;
    let teams_b: HashSet<Uuid> = teams_repo::find_teams_by_user_id(pool, user_b)
        .await?
        .into_iter()
        .map(|team| team.id)
        .collect();

    Ok(teams_a.iter().any(|team| teams_b.contains(&team.id)))
}

async fn require_shared_team(pool: &PgPool, user_a: Uuid, user_b: Uuid) -> Result<(), AppError> {
    if shares_team(pool, user_a, user_b).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "You do not share a team with this user".into(),
        ))
    }
}
