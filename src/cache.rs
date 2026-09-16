//! In-memory cache for gateway and REST models.
//!
//! The cache keeps gateway-backed state coherent and is shared cheaply between
//! tasks. Accessors return cloned models so callers never hold a lock across
//! awaits. Partial gateway updates are merged into cached models when possible;
//! if a partial update cannot be applied safely, the stale entry is evicted so
//! the next cache-first client lookup falls back to REST.

use std::{collections::HashMap, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::debug;

use crate::models::{
    Channel, GatewayEvent, Id, Member, MemberId, MemberList, Message, Role, Server, ServerBans,
    User,
};

/// Shared in-memory cache for users, servers, channels, members, roles, and messages.
#[derive(Clone, Debug, Default)]
pub struct Cache {
    inner: Arc<RwLock<CacheInner>>,
}

#[derive(Clone, Debug, Default)]
struct CacheInner {
    users: HashMap<Id, User>,
    servers: HashMap<Id, Server>,
    channels: HashMap<Id, Channel>,
    members: HashMap<MemberId, Member>,
    messages: HashMap<Id, Message>,
}

/// Snapshot of cache sizes for diagnostics and tests.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CacheCounts {
    /// Number of cached users.
    pub users: usize,
    /// Number of cached servers.
    pub servers: usize,
    /// Number of roles nested in cached servers.
    pub roles: usize,
    /// Number of cached channels.
    pub channels: usize,
    /// Number of cached server members.
    pub members: usize,
    /// Number of cached messages.
    pub messages: usize,
}

impl CacheInner {
    fn apply_event(&mut self, event: &GatewayEvent) {
        match event {
            GatewayEvent::Bulk { v } => {
                for event in v {
                    self.apply_event(event);
                }
            }
            GatewayEvent::Ready(ready) => {
                // Ready is the authoritative snapshot for a new gateway session.
                *self = Self::default();

                for value in &ready.users {
                    if let Some(user) = decode_ready::<User>(value, "user") {
                        self.users.insert(user.id.clone(), user);
                    }
                }
                for value in &ready.servers {
                    if let Some(mut server) = decode_ready::<Server>(value, "server") {
                        normalize_server_roles(&mut server);
                        self.servers.insert(server.id.clone(), server);
                    }
                }
                for value in &ready.channels {
                    if let Some(channel) = decode_ready::<Channel>(value, "channel") {
                        self.channels.insert(channel.id().to_owned(), channel);
                    }
                }
                for value in &ready.members {
                    if let Some(member) = decode_ready::<Member>(value, "member") {
                        self.members.insert(member.id.clone(), member);
                    }
                }
            }
            GatewayEvent::Message(message) => {
                self.messages.insert(message.id.clone(), message.clone());
            }
            GatewayEvent::MessageUpdate(event) => {
                let evict = self
                    .messages
                    .get_mut(&event.id)
                    .map(|message| !merge_partial(message, &event.data, &[]))
                    .unwrap_or(false);
                if evict {
                    self.messages.remove(&event.id);
                }
            }
            GatewayEvent::MessageAppend(event) => {
                // Append payloads have field-specific semantics. Evict rather than
                // risk serving a partially updated message.
                self.messages.remove(&event.id);
            }
            GatewayEvent::MessageDelete(event) => {
                self.messages.remove(&event.id);
            }
            GatewayEvent::MessageReact(event) | GatewayEvent::MessageUnreact(event) => {
                self.messages.remove(&event.id);
            }
            GatewayEvent::MessageRemoveReaction(event) => {
                self.messages.remove(&event.id);
            }
            GatewayEvent::ChannelCreate(channel) => {
                self.channels.insert(channel.id().to_owned(), channel.clone());
            }
            GatewayEvent::ChannelUpdate(event) => {
                let evict = self
                    .channels
                    .get_mut(&event.id)
                    .map(|channel| !merge_partial(channel, &event.data, &event.clear))
                    .unwrap_or(false);
                if evict {
                    self.channels.remove(&event.id);
                }
            }
            GatewayEvent::ChannelDelete(event) => {
                self.channels.remove(&event.id);
            }
            GatewayEvent::ChannelGroupJoin(event) | GatewayEvent::ChannelGroupLeave(event) => {
                // Group membership is embedded in the channel object. Refetch it
                // on demand rather than reconstructing variant-specific state.
                self.channels.remove(&event.id);
            }
            GatewayEvent::ServerCreate(server) => {
                let mut server = server.clone();
                normalize_server_roles(&mut server);
                self.servers.insert(server.id.clone(), server);
            }
            GatewayEvent::ServerUpdate(event) => {
                let evict = self
                    .servers
                    .get_mut(&event.id)
                    .map(|server| !merge_partial(server, &event.data, &event.clear))
                    .unwrap_or(false);
                if evict {
                    self.servers.remove(&event.id);
                }
            }
            GatewayEvent::ServerDelete(event) => {
                if let Some(server) = self.servers.remove(&event.id) {
                    for channel_id in server.channels {
                        self.channels.remove(&channel_id);
                    }
                }
                self.members.retain(|id, _| id.server.as_str() != event.id.as_str());
            }
            GatewayEvent::ServerMemberUpdate(event) => {
                let evict = self
                    .members
                    .get_mut(&event.id)
                    .map(|member| !merge_partial(member, &event.data, &event.clear))
                    .unwrap_or(false);
                if evict {
                    self.members.remove(&event.id);
                }
            }
            GatewayEvent::ServerMemberJoin(event) => {
                let mut member = event.member.clone();
                // Be tolerant of older join payloads while keeping one canonical key.
                member.id.server = event.id.clone();
                member.id.user = event.user.clone();
                self.members.insert(member.id.clone(), member);
            }
            GatewayEvent::ServerMemberLeave(event) => {
                self.members.remove(&MemberId {
                    server: event.id.clone(),
                    user: event.user.clone(),
                });
            }
            GatewayEvent::ServerRoleUpdate(event) => {
                let mut evict_server = false;

                if let Some(server) = self.servers.get_mut(&event.id) {
                    if let Some(role) = server.roles.get_mut(&event.role_id) {
                        if merge_partial(role, &event.data, &event.clear) {
                            role.id = event.role_id.clone();
                        } else {
                            evict_server = true;
                        }
                    } else {
                        let mut role_value = event.data.clone();
                        apply_clear_fields(&mut role_value, &event.clear);
                        if let Value::Object(object) = &mut role_value {
                            object.insert("_id".into(), Value::String(event.role_id.clone()));
                        }

                        match serde_json::from_value::<Role>(role_value) {
                            Ok(mut role) => {
                                role.id = event.role_id.clone();
                                server.roles.insert(event.role_id.clone(), role);
                            }
                            Err(error) => {
                                debug!(
                                    server_id = %event.id,
                                    role_id = %event.role_id,
                                    %error,
                                    "could not materialize new role from gateway update; evicting cached server"
                                );
                                evict_server = true;
                            }
                        }
                    }
                }

                if evict_server {
                    self.servers.remove(&event.id);
                }
            }
            GatewayEvent::ServerRoleRanksUpdate(event) => {
                self.apply_role_ranks(&event.id, &event.ranks);
            }
            GatewayEvent::ServerRoleDelete(event) => {
                if let Some(server) = self.servers.get_mut(&event.id) {
                    server.roles.remove(&event.role_id);
                }
                for (id, member) in &mut self.members {
                    if id.server.as_str() == event.id.as_str() {
                        member.roles.retain(|role_id| role_id != &event.role_id);
                    }
                }
            }
            GatewayEvent::UserUpdate(event) => {
                let evict = self
                    .users
                    .get_mut(&event.id)
                    .map(|user| !merge_partial(user, &event.data, &event.clear))
                    .unwrap_or(false);
                if evict {
                    self.users.remove(&event.id);
                }
            }
            GatewayEvent::UserRelationship(event) => {
                self.users.insert(event.user.id.clone(), event.user.clone());
            }
            GatewayEvent::UserPlatformWipe(event) => {
                self.users.remove(&event.user_id);
                self.members.retain(|id, _| id.user.as_str() != event.user_id.as_str());
            }
            _ => {}
        }
    }

    fn apply_role_ranks(&mut self, server_id: &str, ranks: &[Id]) {
        let mut evict_server = false;

        if let Some(server) = self.servers.get_mut(server_id) {
            let complete = ranks.len() == server.roles.len()
                && ranks.iter().all(|role_id| server.roles.contains_key(role_id));

            if complete {
                for (rank, role_id) in ranks.iter().enumerate() {
                    if let Some(role) = server.roles.get_mut(role_id) {
                        role.rank = Some(rank as i64);
                    }
                }
            } else {
                // Rank events are documented as a complete ordering. If our
                // snapshot disagrees, it is already stale; refresh on demand.
                evict_server = true;
            }
        }

        if evict_server {
            self.servers.remove(server_id);
        }
    }
}

impl Cache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the cache from a gateway event.
    pub async fn update_from_event(&self, event: &GatewayEvent) {
        self.inner.write().await.apply_event(event);
    }

    /// Insert every cacheable model contained in a server member list.
    pub async fn insert_member_list(&self, member_list: &MemberList) {
        let mut inner = self.inner.write().await;
        for user in &member_list.users {
            inner.users.insert(user.id.clone(), user.clone());
        }
        for member in &member_list.members {
            inner.members.insert(member.id.clone(), member.clone());
        }
    }

    /// Insert every cacheable model contained in a server bans response.
    pub async fn insert_server_bans(&self, server_bans: &ServerBans) {
        self.insert_users(server_bans.users.clone()).await;
    }

    /// Insert or replace a user in the cache.
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
    pub async fn insert_server(&self, mut server: Server) -> Option<Server> {
        normalize_server_roles(&mut server);
        self.inner.write().await.servers.insert(server.id.clone(), server)
    }

    /// Insert or replace several servers.
    pub async fn insert_servers(&self, servers: impl IntoIterator<Item = Server>) {
        let mut inner = self.inner.write().await;
        for mut server in servers {
            normalize_server_roles(&mut server);
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

    /// Remove a cached server and dependent member/channel state.
    pub async fn remove_server(&self, id: impl AsRef<str>) -> Option<Server> {
        let id = id.as_ref();
        let mut inner = self.inner.write().await;
        let removed = inner.servers.remove(id);
        inner.members.retain(|member_id, _| member_id.server.as_str() != id);
        if let Some(server) = &removed {
            for channel_id in &server.channels {
                inner.channels.remove(channel_id);
            }
        }
        removed
    }

    /// Fetch a role from a cached server.
    pub async fn role(&self, server_id: impl AsRef<str>, role_id: impl AsRef<str>) -> Option<Role> {
        self.inner
            .read()
            .await
            .servers
            .get(server_id.as_ref())
            .and_then(|server| server.roles.get(role_id.as_ref()))
            .cloned()
    }

    /// Fetch all cached roles for a server.
    pub async fn roles(&self, server_id: impl AsRef<str>) -> Vec<Role> {
        self.inner
            .read()
            .await
            .servers
            .get(server_id.as_ref())
            .map(|server| server.roles.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Insert or replace a role inside an already cached server.
    pub async fn insert_role(
        &self,
        server_id: impl AsRef<str>,
        role_id: impl Into<Id>,
        mut role: Role,
    ) -> Option<Role> {
        let server_id = server_id.as_ref();
        let role_id = role_id.into();
        role.id = role_id.clone();
        self.inner
            .write()
            .await
            .servers
            .get_mut(server_id)
            .and_then(|server| server.roles.insert(role_id, role))
    }

    /// Apply a complete role ordering to an already cached server.
    ///
    /// If the ordering does not match the cached role set, the stale server
    /// snapshot is evicted so the next lookup refreshes it from REST.
    pub async fn set_role_ranks(&self, server_id: impl AsRef<str>, ranks: &[Id]) {
        self.inner
            .write()
            .await
            .apply_role_ranks(server_id.as_ref(), ranks);
    }

    /// Remove a role from a cached server and from cached member assignments.
    pub async fn remove_role(
        &self,
        server_id: impl AsRef<str>,
        role_id: impl AsRef<str>,
    ) -> Option<Role> {
        let server_id = server_id.as_ref();
        let role_id = role_id.as_ref();
        let mut inner = self.inner.write().await;
        let removed = inner
            .servers
            .get_mut(server_id)
            .and_then(|server| server.roles.remove(role_id));
        for (id, member) in &mut inner.members {
            if id.server.as_str() == server_id {
                member.roles.retain(|id| id != role_id);
            }
        }
        removed
    }

    /// Insert or replace a channel in the cache.
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

    /// Insert or replace a server member in the cache.
    pub async fn insert_member(&self, member: Member) -> Option<Member> {
        self.inner
            .write()
            .await
            .members
            .insert(member.id.clone(), member)
    }

    /// Insert or replace several server members.
    pub async fn insert_members(&self, members: impl IntoIterator<Item = Member>) {
        let mut inner = self.inner.write().await;
        for member in members {
            inner.members.insert(member.id.clone(), member);
        }
    }

    /// Fetch a cached server member by server and user ID.
    pub async fn member(
        &self,
        server_id: impl AsRef<str>,
        user_id: impl AsRef<str>,
    ) -> Option<Member> {
        self.inner
            .read()
            .await
            .members
            .get(&MemberId {
                server: server_id.as_ref().to_owned(),
                user: user_id.as_ref().to_owned(),
            })
            .cloned()
    }

    /// Fetch all cached members for a server.
    pub async fn members(&self, server_id: impl AsRef<str>) -> Vec<Member> {
        let server_id = server_id.as_ref();
        self.inner
            .read()
            .await
            .members
            .iter()
            .filter(|(id, _)| id.server.as_str() == server_id)
            .map(|(_, member)| member.clone())
            .collect()
    }

    /// Remove a cached server member by server and user ID.
    pub async fn remove_member(
        &self,
        server_id: impl AsRef<str>,
        user_id: impl AsRef<str>,
    ) -> Option<Member> {
        self.inner.write().await.members.remove(&MemberId {
            server: server_id.as_ref().to_owned(),
            user: user_id.as_ref().to_owned(),
        })
    }

    /// Insert or replace a message in the cache.
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
            roles: inner.servers.values().map(|server| server.roles.len()).sum(),
            channels: inner.channels.len(),
            members: inner.members.len(),
            messages: inner.messages.len(),
        }
    }

    /// Clear all cached values.
    pub async fn clear(&self) {
        *self.inner.write().await = CacheInner::default();
    }
}

fn decode_ready<T: DeserializeOwned>(value: &Value, kind: &str) -> Option<T> {
    match serde_json::from_value(value.clone()) {
        Ok(value) => Some(value),
        Err(error) => {
            debug!(%error, model = kind, "failed to decode model from Ready event");
            None
        }
    }
}

fn normalize_server_roles(server: &mut Server) {
    for (role_id, role) in &mut server.roles {
        if role.id.is_empty() {
            role.id = role_id.clone();
        }
    }
}

fn merge_partial<T>(current: &mut T, data: &Value, clear: &[String]) -> bool
where
    T: DeserializeOwned + Serialize,
{
    let Ok(mut value) = serde_json::to_value(&*current) else {
        return false;
    };

    let (Value::Object(base), Value::Object(changes)) = (&mut value, data) else {
        return false;
    };

    for (key, value) in changes {
        base.insert(key.clone(), value.clone());
    }
    for field in clear {
        base.remove(&event_field_to_json(field));
    }

    match serde_json::from_value(value) {
        Ok(updated) => {
            *current = updated;
            true
        }
        Err(error) => {
            debug!(%error, "failed to merge partial gateway update into cached model");
            false
        }
    }
}

fn apply_clear_fields(value: &mut Value, clear: &[String]) {
    if let Value::Object(object) = value {
        for field in clear {
            object.remove(&event_field_to_json(field));
        }
    }
}

fn event_field_to_json(field: &str) -> String {
    let mut output = String::with_capacity(field.len() + 4);
    for (index, character) in field.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index != 0 {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    output
}
