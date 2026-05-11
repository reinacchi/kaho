//! In-memory cache for gateway models.
//!
//! The cache is intentionally small and type-safe: it stores the latest copy of
//! entities seen by the client and can be shared cheaply between tasks.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::models::{Channel, GatewayEvent, Id, Message, Server, User};

/// Represents a cache value used by the Stoat API models and endpoints.
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
    /// Calls the Stoat API or client internals to new for this resource.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calls the Stoat API or client internals to update from event for this resource.
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

    /// Insert or replace a user in the cache.
    ///
    /// Returns the previously cached user with the same ID, if one existed.
    pub async fn insert_user(&self, user: User) -> Option<User> {
        self.inner.write().await.users.insert(user.id.clone(), user)
    }

    /// Fetch a cached user by ID.
    ///
    /// Returns a cloned user so callers do not hold the cache read lock.
    pub async fn user(&self, id: impl AsRef<str>) -> Option<User> {
        self.inner.read().await.users.get(id.as_ref()).cloned()
    }

    /// Insert or replace a server in the cache.
    ///
    /// Returns the previously cached server with the same ID, if one existed.
    pub async fn insert_server(&self, server: Server) -> Option<Server> {
        self.inner.write().await.servers.insert(server.id.clone(), server)
    }

    /// Calls the Stoat API or client internals to server for this resource.
    pub async fn server(&self, id: impl AsRef<str>) -> Option<Server> {
        self.inner.read().await.servers.get(id.as_ref()).cloned()
    }

    /// Insert or replace a channel in the cache.
    ///
    /// Returns the previously cached channel with the same ID, if one existed.
    pub async fn insert_channel(&self, channel: Channel) -> Option<Channel> {
        let id = channel.id().to_owned();
        self.inner.write().await.channels.insert(id, channel)
    }

    /// Calls the Stoat API or client internals to channel for this resource.
    pub async fn channel(&self, id: impl AsRef<str>) -> Option<Channel> {
        self.inner.read().await.channels.get(id.as_ref()).cloned()
    }

    /// Insert or replace a message in the cache.
    ///
    /// Returns the previously cached message with the same ID, if one existed.
    pub async fn insert_message(&self, message: Message) -> Option<Message> {
        self.inner.write().await.messages.insert(message.id.clone(), message)
    }

    /// Calls the Stoat API or client internals to message for this resource.
    pub async fn message(&self, id: impl AsRef<str>) -> Option<Message> {
        self.inner.read().await.messages.get(id.as_ref()).cloned()
    }

    /// Calls the Stoat API or client internals to clear for this resource.
    pub async fn clear(&self) {
        *self.inner.write().await = CacheInner::default();
    }
}
