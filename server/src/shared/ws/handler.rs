use std::collections::HashMap;

use axum::{
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

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

    let session = sqlx::query!(
        "SELECT s.user_id, u.username FROM sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.id = $1 AND s.expires_at > now()",
        token
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::Unauthorized)?
    .ok_or(AppError::Unauthorized)?;

    let username = session.username;
    let hub = state.hub.clone();

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, hub, username)))
}

async fn handle_socket(socket: WebSocket, hub: Hub, username: String) {
    let client_id = Uuid::now_v7();
    let (tx, mut rx) = mpsc::unbounded_channel();

    hub.connect(client_id, tx).await;

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
                    if let Ok(event) = serde_json::from_str::<ClientMessage>(&text)
                        && matches!(event, ClientMessage::Ping)
                    {
                        hub_clone.send_to(client_id, &WsEvent::Pong).await;
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
