use std::collections::HashMap;

use axum::{
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::features::auth::repo as auth_repo;
use crate::features::incidents::repo as incidents_repo;
use crate::features::teams::repo as teams_repo;
use crate::shared::error::AppError;
use crate::shared::ws::{
    event::{ClientMessage, WsEvent},
    hub::Hub,
};
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, AppError> {
    let token = params
        .get("token")
        .and_then(|t| Uuid::parse_str(t).ok())
        .ok_or(AppError::Unauthorized)?;

    let session = auth_repo::find_session_with_username(&state.db, token)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if session.is_expired() {
        return Err(AppError::Unauthorized);
    }

    let team_ids = teams_repo::find_teams_by_user_id(&state.db, session.user_id)
        .await?
        .into_iter()
        .map(|team| team.id)
        .collect();

    let username = session.username;
    let hub = state.hub.clone();
    let db = state.db.clone();

    Ok(ws.on_upgrade(move |socket| {
        handle_socket(socket, hub, db, session.user_id, username, team_ids)
    }))
}

async fn can_watch_incident(db: &PgPool, user_id: Uuid, incident_id: Uuid) -> bool {
    let Ok(Some(incident)) = incidents_repo::find_incident_by_id(db, incident_id).await else {
        return false;
    };

    matches!(
        teams_repo::find_member(db, incident.team_id, user_id).await,
        Ok(Some(_))
    )
}

async fn handle_socket(
    socket: WebSocket,
    hub: Hub,
    db: PgPool,
    user_id: Uuid,
    username: String,
    team_ids: Vec<Uuid>,
) {
    let client_id = Uuid::now_v7();
    let (tx, mut rx) = mpsc::unbounded_channel();

    hub.connect(client_id, tx, username.clone(), team_ids).await;

    let (mut sink, mut stream) = socket.split();

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let hub_clone = hub.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(text) => {
                    let Ok(event) = serde_json::from_str::<ClientMessage>(&text) else {
                        continue;
                    };
                    match event {
                        ClientMessage::Ping => {
                            hub_clone.send_to(client_id, &WsEvent::Pong).await;
                        }
                        ClientMessage::Watch { incident_id } => {
                            if !can_watch_incident(&db, user_id, incident_id).await {
                                continue;
                            }
                            let watchers = hub_clone.watch_incident(client_id, incident_id).await;
                            hub_clone
                                .broadcast_to_watchers(
                                    incident_id,
                                    &WsEvent::PresenceUpdate {
                                        incident_id,
                                        watchers,
                                    },
                                )
                                .await;
                        }
                        ClientMessage::Unwatch { incident_id } => {
                            let watchers = hub_clone.unwatch_incident(client_id, incident_id).await;
                            hub_clone
                                .broadcast_to_watchers(
                                    incident_id,
                                    &WsEvent::PresenceUpdate {
                                        incident_id,
                                        watchers,
                                    },
                                )
                                .await;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    hub.disconnect(client_id).await;
    tracing::info!("client disconnected: {}", username);
}
