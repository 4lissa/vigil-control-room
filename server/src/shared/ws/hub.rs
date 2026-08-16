use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::shared::ws::event::WsEvent;

pub type ClientSender = mpsc::UnboundedSender<Message>;

#[derive(Debug, Default)]
struct HubInner {
    clients: HashMap<Uuid, ClientSender>,
    team_members: HashMap<Uuid, HashSet<Uuid>>,
    incident_watchers: HashMap<Uuid, HashSet<Uuid>>,
    client_teams: HashMap<Uuid, HashSet<Uuid>>,
    client_username: HashMap<Uuid, String>,
}

#[derive(Debug, Clone, Default)]
pub struct Hub {
    inner: Arc<RwLock<HubInner>>,
}

impl Hub {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn connect(
        &self,
        client_id: Uuid,
        sender: ClientSender,
        username: String,
        team_ids: Vec<Uuid>,
    ) {
        let mut inner = self.inner.write().await;
        inner.clients.insert(client_id, sender);
        inner.client_username.insert(client_id, username);

        let teams: HashSet<Uuid> = team_ids.iter().cloned().collect();
        inner.client_teams.insert(client_id, teams);

        for team_id in team_ids {
            inner
                .team_members
                .entry(team_id)
                .or_default()
                .insert(client_id);
        }
    }

    pub async fn disconnect(&self, client_id: Uuid) {
        let mut inner = self.inner.write().await;
        inner.clients.remove(&client_id);
        inner.client_username.remove(&client_id);

        if let Some(teams) = inner.client_teams.remove(&client_id) {
            for team_id in teams {
                if let Some(members) = inner.team_members.get_mut(&team_id) {
                    members.remove(&client_id);
                    if members.is_empty() {
                        inner.team_members.remove(&team_id);
                    }
                }
            }
        }

        let watched: Vec<Uuid> = inner
            .incident_watchers
            .iter()
            .filter(|(_, watchers)| watchers.contains(&client_id))
            .map(|(id, _)| *id)
            .collect();

        for incident_id in watched {
            if let Some(watchers) = inner.incident_watchers.get_mut(&incident_id) {
                watchers.remove(&client_id);
            }
        }
    }

    pub async fn watch_incident(&self, client_id: Uuid, incident_id: Uuid) -> Vec<String> {
        let mut inner = self.inner.write().await;
        inner
            .incident_watchers
            .entry(incident_id)
            .or_default()
            .insert(client_id);

        self.watcher_usernames(&inner, incident_id)
    }

    pub async fn unwatch_incident(&self, client_id: Uuid, incident_id: Uuid) -> Vec<String> {
        let mut inner = self.inner.write().await;
        if let Some(watchers) = inner.incident_watchers.get_mut(&incident_id) {
            watchers.remove(&client_id);
        }

        self.watcher_usernames(&inner, incident_id)
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

    pub async fn broadcast_to_team(&self, team_id: Uuid, event: &WsEvent) {
        let Ok(msg) = serde_json::to_string(event) else {
            return;
        };
        let inner = self.inner.read().await;
        let Some(client_ids) = inner.team_members.get(&team_id) else {
            return;
        };
        for client_id in client_ids {
            if let Some(sender) = inner.clients.get(client_id) {
                let _ = sender.send(Message::Text(msg.clone().into()));
            }
        }
    }

    pub async fn broadcast_to_watchers(&self, incident_id: Uuid, event: &WsEvent) {
        let Ok(msg) = serde_json::to_string(event) else {
            return;
        };
        let inner = self.inner.read().await;
        let Some(client_ids) = inner.incident_watchers.get(&incident_id) else {
            return;
        };
        for client_id in client_ids {
            if let Some(sender) = inner.clients.get(client_id) {
                let _ = sender.send(Message::Text(msg.clone().into()));
            }
        }
    }

    fn watcher_usernames(&self, inner: &HubInner, incident_id: Uuid) -> Vec<String> {
        inner
            .incident_watchers
            .get(&incident_id)
            .map(|watchers| {
                watchers
                    .iter()
                    .filter_map(|id| inner.client_username.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_adds_client_to_team_room() {
        let hub = Hub::new();
        let client_id = Uuid::now_v7();
        let team_id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel();

        hub.connect(client_id, tx, "alissa".into(), vec![team_id])
            .await;

        let inner = hub.inner.read().await;
        assert!(inner.clients.contains_key(&client_id));
        assert!(
            inner
                .team_members
                .get(&team_id)
                .unwrap()
                .contains(&client_id)
        );
    }

    #[tokio::test]
    async fn disconnect_removes_client_from_team_room() {
        let hub = Hub::new();
        let client_id = Uuid::now_v7();
        let team_id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel();

        hub.connect(client_id, tx, "alissa".into(), vec![team_id])
            .await;
        hub.disconnect(client_id).await;

        let inner = hub.inner.read().await;
        assert!(!inner.clients.contains_key(&client_id));
        assert!(!inner.team_members.contains_key(&team_id));
    }

    #[tokio::test]
    async fn broadcast_to_team_delivers_only_to_team_members() {
        let hub = Hub::new();
        let team_id = Uuid::now_v7();
        let other_team_id = Uuid::now_v7();

        let client_a = Uuid::now_v7();
        let client_b = Uuid::now_v7();
        let (tx_a, mut rx_a) = mpsc::unbounded_channel();
        let (tx_b, mut rx_b) = mpsc::unbounded_channel();

        hub.connect(client_a, tx_a, "alissa".into(), vec![team_id])
            .await;
        hub.connect(client_b, tx_b, "bob".into(), vec![other_team_id])
            .await;

        hub.broadcast_to_team(team_id, &WsEvent::Pong).await;

        assert!(rx_a.try_recv().is_ok());
        assert!(rx_b.try_recv().is_err());
    }

    #[tokio::test]
    async fn watch_incident_returns_watcher_usernames() {
        let hub = Hub::new();
        let client_id = Uuid::now_v7();
        let incident_id = Uuid::now_v7();
        let team_id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel();

        hub.connect(client_id, tx, "alissa".into(), vec![team_id])
            .await;
        let watchers = hub.watch_incident(client_id, incident_id).await;

        assert_eq!(watchers, vec!["alissa"]);
    }

    #[tokio::test]
    async fn broadcast_to_watchers_delivers_only_to_watchers() {
        let hub = Hub::new();
        let team_id = Uuid::now_v7();
        let incident_id = Uuid::now_v7();
        let other_incident_id = Uuid::now_v7();

        let watcher = Uuid::now_v7();
        let bystander = Uuid::now_v7();
        let (tx_watcher, mut rx_watcher) = mpsc::unbounded_channel();
        let (tx_bystander, mut rx_bystander) = mpsc::unbounded_channel();

        hub.connect(watcher, tx_watcher, "alissa".into(), vec![team_id])
            .await;
        hub.connect(bystander, tx_bystander, "bob".into(), vec![team_id])
            .await;
        hub.watch_incident(watcher, incident_id).await;
        hub.watch_incident(bystander, other_incident_id).await;

        hub.broadcast_to_watchers(incident_id, &WsEvent::Pong).await;

        assert!(rx_watcher.try_recv().is_ok());
        assert!(rx_bystander.try_recv().is_err());
    }

    #[tokio::test]
    async fn unwatch_incident_removes_watcher() {
        let hub = Hub::new();
        let client_id = Uuid::now_v7();
        let incident_id = Uuid::now_v7();
        let team_id = Uuid::now_v7();
        let (tx, _rx) = mpsc::unbounded_channel();

        hub.connect(client_id, tx, "alissa".into(), vec![team_id])
            .await;
        hub.watch_incident(client_id, incident_id).await;
        let watchers = hub.unwatch_incident(client_id, incident_id).await;

        assert!(watchers.is_empty());
    }
}
