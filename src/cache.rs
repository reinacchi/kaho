//! In-memory cache for gateway and REST models.
//!
//! The cache is intentionally small and type-safe: it stores the latest copy of
//! entities seen by the client and can be shared cheaply between tasks. All
//! accessors return cloned models so callers never hold a lock across awaits.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::models::{Channel, GatewayEvent, Id, MemberList, Message, Server, ServerBans, User};

/// Shared in-memory cache for users, servers, channels, and messages.
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

/// Snapshot of cache sizes for diagnostics and tests.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CacheCounts {
    /// Number of cached users.
    pub users: usize,
    /// Number of cached servers.
    pub servers: usize,
    /// Number of cached channels.
    pub channels: usize,
    /// Number of cached messages.
    pub messages: usize,
}

impl Cache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the cache from a gateway event.
    pub async fn update_from_event(&self, event: &GatewayEvent) {
        match event {
            GatewayEvent::Message(message) => {
                self.insert_message(message.clone()).await;
            }
            GatewayEvent::ServerCreate(server) => {
                self.insert_server(server.clone()).await;
            }
            _ => {}
        }
    }

    /// Insert every cacheable model contained in a server member list.
    pub async fn insert_member_list(&self, member_list: &MemberList) {
        self.insert_users(member_list.users.clone()).await;
    }

    /// Insert every cacheable model contained in a server bans response.
    pub async fn insert_server_bans(&self, server_bans: &ServerBans) {
        self.insert_users(server_bans.users.clone()).await;
    }

    /// Insert or replace a user in the cache.
    ///
    /// Returns the previously cached user with the same ID, if one existed.
    pub async fn insert_user(&self, user: User) -> Option<User> {
        self.inner.write().await.users.insert(user.id.clone(), user)
    }

    /// Insert or replace several users.
    pub async fn insert_users(&self, users: impl IntoIterator<Item = User>) {
        let mut inner = self.inner.write().await;
        for user in users {
            inner.users.insert(user.id.clone(), user);
        }
    }

    /// Fetch a cached user by ID.
    pub async fn user(&self, id: impl AsRef<str>) -> Option<User> {
        self.inner.read().await.users.get(id.as_ref()).cloned()
    }

    /// Fetch all cached users.
    pub async fn users(&self) -> Vec<User> {
        self.inner.read().await.users.values().cloned().collect()
    }

    /// Remove a cached user by ID.
    pub async fn remove_user(&self, id: impl AsRef<str>) -> Option<User> {
        self.inner.write().await.users.remove(id.as_ref())
    }

    /// Insert or replace a server in the cache.
    ///
    /// Returns the previously cached server with the same ID, if one existed.
    pub async fn insert_server(&self, server: Server) -> Option<Server> {
        self.inner.write().await.servers.insert(server.id.clone(), server)
    }

    /// Insert or replace several servers.
    pub async fn insert_servers(&self, servers: impl IntoIterator<Item = Server>) {
        let mut inner = self.inner.write().await;
        for server in servers {
            inner.servers.insert(server.id.clone(), server);
        }
    }

    /// Fetch a cached server by ID.
    pub async fn server(&self, id: impl AsRef<str>) -> Option<Server> {
        self.inner.read().await.servers.get(id.as_ref()).cloned()
    }

    /// Fetch all cached servers.
    pub async fn servers(&self) -> Vec<Server> {
        self.inner.read().await.servers.values().cloned().collect()
    }

    /// Remove a cached server by ID.
    pub async fn remove_server(&self, id: impl AsRef<str>) -> Option<Server> {
        self.inner.write().await.servers.remove(id.as_ref())
    }

    /// Insert or replace a channel in the cache.
    ///
    /// Returns the previously cached channel with the same ID, if one existed.
    pub async fn insert_channel(&self, channel: Channel) -> Option<Channel> {
        let id = channel.id().to_owned();
        self.inner.write().await.channels.insert(id, channel)
    }

    /// Insert or replace several channels.
    pub async fn insert_channels(&self, channels: impl IntoIterator<Item = Channel>) {
        let mut inner = self.inner.write().await;
        for channel in channels {
            let id = channel.id().to_owned();
            inner.channels.insert(id, channel);
        }
    }

    /// Fetch a cached channel by ID.
    pub async fn channel(&self, id: impl AsRef<str>) -> Option<Channel> {
        self.inner.read().await.channels.get(id.as_ref()).cloned()
    }

    /// Fetch all cached channels.
    pub async fn channels(&self) -> Vec<Channel> {
        self.inner.read().await.channels.values().cloned().collect()
    }

    /// Remove a cached channel by ID.
    pub async fn remove_channel(&self, id: impl AsRef<str>) -> Option<Channel> {
        self.inner.write().await.channels.remove(id.as_ref())
    }

    /// Insert or replace a message in the cache.
    ///
    /// Returns the previously cached message with the same ID, if one existed.
    pub async fn insert_message(&self, message: Message) -> Option<Message> {
        self.inner.write().await.messages.insert(message.id.clone(), message)
    }

    /// Insert or replace several messages.
    pub async fn insert_messages(&self, messages: impl IntoIterator<Item = Message>) {
        let mut inner = self.inner.write().await;
        for message in messages {
            inner.messages.insert(message.id.clone(), message);
        }
    }

    /// Fetch a cached message by ID.
    pub async fn message(&self, id: impl AsRef<str>) -> Option<Message> {
        self.inner.read().await.messages.get(id.as_ref()).cloned()
    }

    /// Fetch all cached messages.
    pub async fn messages(&self) -> Vec<Message> {
        self.inner.read().await.messages.values().cloned().collect()
    }

    /// Remove a cached message by ID.
    pub async fn remove_message(&self, id: impl AsRef<str>) -> Option<Message> {
        self.inner.write().await.messages.remove(id.as_ref())
    }

    /// Return the number of cached values per model type.
    pub async fn counts(&self) -> CacheCounts {
        let inner = self.inner.read().await;
        CacheCounts {
            users: inner.users.len(),
            servers: inner.servers.len(),
            channels: inner.channels.len(),
            messages: inner.messages.len(),
        }
    }

    /// Clear all cached values.
    pub async fn clear(&self) {
        *self.inner.write().await = CacheInner::default();
    }
}
