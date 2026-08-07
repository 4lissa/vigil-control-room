use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::shared::ws::event::WsEvent;

pub type ClientSender = mpsc::UnboundedSender<Message>;

#[derive(Debug, Default)]
struct HubInner {
    clients: HashMap<Uuid, ClientSender>,
}

#[derive(Debug, Clone, Default)]
pub struct Hub {
    inner: Arc<RwLock<HubInner>>,
}

impl Hub {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn connect(&self, client_id: Uuid, sender: ClientSender) {
        let mut inner = self.inner.write().await;
        inner.clients.insert(client_id, sender);
    }

    pub async fn disconnect(&self, client_id: Uuid) {
        let mut inner = self.inner.write().await;
        inner.clients.remove(&client_id);
    }

    pub async fn send_to(&self, client_id: Uuid, event: &WsEvent) {
        let Ok(msg) = serde_json::to_string(event) else {
            return;
        };
        let inner = self.inner.read().await;
        if let Some(sender) = inner.clients.get(&client_id) {
            let _ = sender.send(Message::Text(msg.into()));
        }
    }

    pub async fn broadcast(&self, event: &WsEvent) {
        let Ok(msg) = serde_json::to_string(event) else {
            return;
        };
        let inner = self.inner.read().await;
        for sender in inner.clients.values() {
            let _ = sender.send(Message::Text(msg.clone().into()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_adds_client() {
        let hub = Hub::new();
        let client_id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel();

        hub.connect(client_id, tx).await;

        let inner = hub.inner.read().await;
        assert!(inner.clients.contains_key(&client_id));
    }

    #[tokio::test]
    async fn disconnect_removes_client() {
        let hub = Hub::new();
        let client_id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel();

        hub.connect(client_id, tx).await;
        hub.disconnect(client_id).await;

        let inner = hub.inner.read().await;
        assert!(!inner.clients.contains_key(&client_id));
    }

    #[tokio::test]
    async fn send_to_delivers_message() {
        let hub = Hub::new();
        let client_id = Uuid::now_v7();
        let (tx, mut rx) = mpsc::unbounded_channel();

        hub.connect(client_id, tx).await;
        hub.send_to(client_id, &WsEvent::Pong).await;

        let msg = rx.try_recv().expect("message should be in channel");
        match msg {
            Message::Text(text) => {
                assert_eq!(text, r#"{"type":"pong"}"#);
            }
            _ => panic!("expected text message"),
        }
    }

    #[tokio::test]
    async fn send_to_unknown_client_does_nothing() {
        let hub = Hub::new();
        let unknown_id = Uuid::now_v7();
        hub.send_to(unknown_id, &WsEvent::Pong).await;
    }

    #[tokio::test]
    async fn broadcast_delivers_to_all_clients() {
        let hub = Hub::new();
        let client_a = Uuid::now_v7();
        let client_b = Uuid::now_v7();
        let (tx_a, mut rx_a) = mpsc::unbounded_channel();
        let (tx_b, mut rx_b) = mpsc::unbounded_channel();

        hub.connect(client_a, tx_a).await;
        hub.connect(client_b, tx_b).await;
        hub.broadcast(&WsEvent::Pong).await;

        assert!(rx_a.try_recv().is_ok());
        assert!(rx_b.try_recv().is_ok());
    }
}
