use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

use crate::features::auth::repo as auth_repo;
use crate::features::messages::dto::{MessageResponse, SendMessageRequest};
use crate::features::messages::service;
use crate::shared::error::{AppError, validation_message};
use crate::shared::middleware::AuthUser;
use crate::shared::ws::event::WsEvent;
use crate::state::AppState;

pub async fn send_message(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(recipient_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    req.validate()
        .map_err(|e| AppError::ValidationError(validation_message(&e)))?;

    let recipient = auth_repo::find_user_by_id(&state.db, recipient_id)
        .await?
        .ok_or(AppError::NotFound("Recipient not found".into()))?;

    let message =
        service::send_message(&state.db, auth_user.user_id, recipient_id, &req.content).await?;

    let event = WsEvent::PrivateMessageReceived {
        from: auth_user.username.clone(),
        to: recipient.username,
        content: message.content.clone(),
        at: message.created_at.unix_timestamp(),
    };

    state.hub.send_to_user(auth_user.user_id, &event).await;
    state.hub.send_to_user(recipient_id, &event).await;

    Ok((StatusCode::CREATED, Json(message.into())))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(other_user_id): Path<Uuid>,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    let messages = service::get_conversation(&state.db, auth_user.user_id, other_user_id).await?;
    Ok(Json(messages.into_iter().map(Into::into).collect()))
}

pub async fn list_conversations(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<MessageResponse>>, AppError> {
    let conversations = service::list_conversations(&state.db, auth_user.user_id).await?;
    Ok(Json(conversations.into_iter().map(Into::into).collect()))
}
