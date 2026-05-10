//! In-memory cache for gateway models.
//!
//! The cache is intentionally small and type-safe: it stores the latest copy of
//! entities seen by the client and can be shared cheaply between tasks.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::models::{Channel, GatewayEvent, Id, Message, Server, User};

/// Shared, concurrent entity cache.
#[derive(Clone, Debug, Default)]
pub struct Cache {
    inner: Arc<RwLock<CacheInner>>,
}

#[derive(Clone, Debug, Default)]
struct CacheInner {
    users: HashMap<Id, User>,
    servers: HashMap<Id, Server>,
    channels: HashMap<Id, Channel>,
    messages: HashMap<Id, Message>,
}

impl Cache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Store entities carried by a gateway event.
    pub async fn update_from_event(&self, event: &GatewayEvent) {
        let mut inner = self.inner.write().await;

        match event {
            GatewayEvent::Message(message) => {
                inner.messages.insert(message.id.clone(), message.clone());
            }
            GatewayEvent::ServerCreate(server) => {
                inner.servers.insert(server.id.clone(), server.clone());
            }
            _ => {}
        }
    }

    pub async fn insert_user(&self, user: User) -> Option<User> {
        self.inner.write().await.users.insert(user.id.clone(), user)
    }

    pub async fn user(&self, id: impl AsRef<str>) -> Option<User> {
        self.inner.read().await.users.get(id.as_ref()).cloned()
    }

    pub async fn insert_server(&self, server: Server) -> Option<Server> {
        self.inner.write().await.servers.insert(server.id.clone(), server)
    }

    pub async fn server(&self, id: impl AsRef<str>) -> Option<Server> {
        self.inner.read().await.servers.get(id.as_ref()).cloned()
    }

    pub async fn insert_channel(&self, channel: Channel) -> Option<Channel> {
        let id = channel.id().to_owned();
        self.inner.write().await.channels.insert(id, channel)
    }

    pub async fn channel(&self, id: impl AsRef<str>) -> Option<Channel> {
        self.inner.read().await.channels.get(id.as_ref()).cloned()
    }

    pub async fn insert_message(&self, message: Message) -> Option<Message> {
        self.inner.write().await.messages.insert(message.id.clone(), message)
    }

    pub async fn message(&self, id: impl AsRef<str>) -> Option<Message> {
        self.inner.read().await.messages.get(id.as_ref()).cloned()
    }

    pub async fn clear(&self) {
        *self.inner.write().await = CacheInner::default();
    }
}
