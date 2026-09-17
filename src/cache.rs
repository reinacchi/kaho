//! In-memory cache for gateway and REST models.
//!
//! The cache keeps gateway-backed state coherent and is shared cheaply between
//! tasks. Accessors return cloned models so callers never hold a lock across
//! awaits. Partial gateway updates are merged into cached models when possible;
//! if a partial update cannot be applied safely, the stale entry is evicted so
//! the next cache-first client lookup falls back to REST.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    mem::replace,
    sync::Arc,
};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value, Value};
use tokio::sync::RwLock;
use tracing::debug;

use crate::models::{
    Channel, GatewayEvent, Id, Member, MemberList, Message, Role, Server, ServerBans, User,
};

/// Shared in-memory cache for users, servers, channels, members, roles, and messages.
#[derive(Clone, Debug, Default)]
pub struct Cache {
    inner: Arc<RwLock<CacheInner>>,
}

const DEFAULT_MESSAGE_CACHE_CAPACITY: usize = 10_000;

#[derive(Clone, Debug)]
struct CacheInner {
    users: HashMap<Id, User>,
    servers: HashMap<Id, Server>,
    channels: HashMap<Id, Channel>,
    members: HashMap<Id, HashMap<Id, Member>>,
    messages: HashMap<Id, Message>,
    message_order: VecDeque<Id>,
    message_capacity: usize,
}

impl Default for CacheInner {
    fn default() -> Self {
        Self::with_message_capacity(DEFAULT_MESSAGE_CACHE_CAPACITY)
    }
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
    fn with_message_capacity(message_capacity: usize) -> Self {
        Self {
            users: HashMap::new(),
            servers: HashMap::new(),
            channels: HashMap::new(),
            members: HashMap::new(),
            messages: HashMap::new(),
            message_order: VecDeque::new(),
            message_capacity,
        }
    }

    fn insert_message(&mut self, message: Message) -> Option<Message> {
        let id = message.id.clone();

        if self.message_capacity == 0 {
            return self.messages.remove(&id);
        }

        // Updating an existing message keeps its original FIFO position. Avoid scanning the
        // entire order queue for every normal Message event; that would make cache ingestion
        // O(cache_size) and could itself create gateway latency under load.
        if let Some(current) = self.messages.get_mut(&id) {
            return Some(replace(current, message));
        }

        self.messages.insert(id.clone(), message);
        self.message_order.push_back(id);
        self.enforce_message_capacity();
        None
    }

    fn remove_message(&mut self, message_id: &str) -> Option<Message> {
        self.message_order.retain(|id| id.as_str() != message_id);
        self.messages.remove(message_id)
    }

    fn remove_messages_for_channel(&mut self, channel_id: &str) {
        self.retain_messages(|message| message.channel.as_str() != channel_id);
    }

    fn insert_channel(&mut self, channel: Channel) -> Option<Channel> {
        let channel_id = channel.id().to_owned();

        if let Some(server_id) = channel_server_id(&channel) {
            if let Some(server) = self.servers.get_mut(server_id) {
                if !server.channels.iter().any(|id| id == &channel_id) {
                    server.channels.push(channel_id.clone());
                }
            }
        }

        self.channels.insert(channel_id, channel)
    }

    fn remove_channel(&mut self, channel_id: &str) -> Option<Channel> {
        let removed = self.channels.remove(channel_id);
        self.remove_messages_for_channel(channel_id);

        for server in self.servers.values_mut() {
            server.channels.retain(|id| id.as_str() != channel_id);
            for category in &mut server.categories {
                category.channels.retain(|id| id.as_str() != channel_id);
            }
        }

        removed
    }

    fn retain_messages(&mut self, mut keep: impl FnMut(&Message) -> bool) {
        self.messages.retain(|_, message| keep(message));
        let messages = &self.messages;
        self.message_order.retain(|id| messages.contains_key(id));
    }

    fn enforce_message_capacity(&mut self) {
        while self.messages.len() > self.message_capacity {
            let Some(oldest) = self.message_order.pop_front() else {
                break;
            };
            self.messages.remove(&oldest);
        }
    }

    fn apply_event(&mut self, event: &GatewayEvent) {
        match event {
            GatewayEvent::Bulk { v } => {
                for event in v {
                    self.apply_event(event);
                }
            }
            GatewayEvent::Ready(ready) => {
                // Ready is the authoritative snapshot for a new gateway session. Preserve the
                // configured message capacity while clearing state from the previous session.
                let message_capacity = self.message_capacity;
                *self = Self::with_message_capacity(message_capacity);

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
                        self.insert_channel(channel);
                    }
                }
                for value in &ready.members {
                    if let Some(member) = decode_ready::<Member>(value, "member") {
                        self.insert_member(member);
                    }
                }
            }
            GatewayEvent::Message(message) => {
                self.insert_message(message.clone());
            }
            GatewayEvent::MessageUpdate(event) => {
                let evict = self
                    .messages
                    .get_mut(&event.id)
                    .map(|message| !merge_partial(message, &event.data, &[]))
                    .unwrap_or(false);
                if evict {
                    self.remove_message(&event.id);
                }
            }
            GatewayEvent::MessageAppend(event) => {
                // Append payloads have field-specific semantics. Evict rather than
                // risk serving a partially updated message.
                self.remove_message(&event.id);
            }
            GatewayEvent::MessageDelete(event) => {
                self.remove_message(&event.id);
            }
            GatewayEvent::MessageReact(event) | GatewayEvent::MessageUnreact(event) => {
                self.remove_message(&event.id);
            }
            GatewayEvent::MessageRemoveReaction(event) => {
                self.remove_message(&event.id);
            }
            GatewayEvent::ChannelCreate(channel) => {
                self.insert_channel(channel.clone());
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
                self.remove_channel(&event.id);
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
                self.remove_server(&event.id);
            }
            GatewayEvent::ServerMemberUpdate(event) => {
                let evict = self
                    .members
                    .get_mut(&event.id.server)
                    .and_then(|members| members.get_mut(&event.id.user))
                    .map(|member| !merge_partial(member, &event.data, &event.clear))
                    .unwrap_or(false);
                if evict {
                    self.remove_member(&event.id.server, &event.id.user);
                }
            }
            GatewayEvent::ServerMemberJoin(event) => {
                let mut member = event.member.clone();
                // Be tolerant of older join payloads while keeping one canonical key.
                member.id.server = event.id.clone();
                member.id.user = event.user.clone();
                self.insert_member(member);
            }
            GatewayEvent::ServerMemberLeave(event) => {
                self.remove_member(&event.id, &event.user);
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

                        match from_value::<Role>(role_value) {
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
                self.remove_role(&event.id, &event.role_id);
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
                self.members.retain(|_, members| {
                    members.remove(&event.user_id);
                    !members.is_empty()
                });

                self.retain_messages(|message| message.author.as_str() != event.user_id.as_str());

                let direct_channels: Vec<Id> = self
                    .channels
                    .iter()
                    .filter_map(|(id, channel)| match channel {
                        Channel::DirectMessage(channel)
                            if channel
                                .recipients
                                .iter()
                                .any(|recipient| recipient == &event.user_id) =>
                        {
                            Some(id.clone())
                        }
                        Channel::SavedMessages(channel)
                            if channel.user.as_str() == event.user_id.as_str() =>
                        {
                            Some(id.clone())
                        }
                        _ => None,
                    })
                    .collect();
                for channel_id in direct_channels {
                    self.channels.remove(&channel_id);
                    self.remove_messages_for_channel(&channel_id);
                }
            }
            _ => {}
        }
    }

    fn insert_member(&mut self, member: Member) -> Option<Member> {
        let server_id = member.id.server.clone();
        let user_id = member.id.user.clone();
        self.members
            .entry(server_id)
            .or_default()
            .insert(user_id, member)
    }

    fn remove_member(&mut self, server_id: &str, user_id: &str) -> Option<Member> {
        let (removed, empty) = {
            let members = self.members.get_mut(server_id)?;
            let removed = members.remove(user_id);
            (removed, members.is_empty())
        };
        if empty {
            self.members.remove(server_id);
        }
        removed
    }

    fn remove_role(&mut self, server_id: &str, role_id: &str) -> Option<Role> {
        let removed = self
            .servers
            .get_mut(server_id)
            .and_then(|server| server.roles.remove(role_id));

        if let Some(members) = self.members.get_mut(server_id) {
            for member in members.values_mut() {
                member.roles.retain(|id| id != role_id);
            }
        }

        removed
    }

    fn remove_server(&mut self, server_id: &str) -> Option<Server> {
        let mut channel_ids: HashSet<Id> = self
            .channels
            .iter()
            .filter_map(|(id, channel)| {
                (channel_server_id(channel) == Some(server_id)).then(|| id.clone())
            })
            .collect();
        let removed = self.servers.remove(server_id);
        if let Some(server) = &removed {
            channel_ids.extend(server.channels.iter().cloned());
        }

        for channel_id in channel_ids {
            self.channels.remove(&channel_id);
            self.remove_messages_for_channel(&channel_id);
        }
        self.members.remove(server_id);
        removed
    }

    fn apply_role_ranks(&mut self, server_id: &str, ranks: &[Id]) {
        let mut evict_server = false;

        if let Some(server) = self.servers.get_mut(server_id) {
            let complete = ranks.len() == server.roles.len()
                && ranks
                    .iter()
                    .all(|role_id| server.roles.contains_key(role_id));

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
            inner.insert_member(member.clone());
        }
    }

    /// Insert every cacheable model contained in a server bans response.
    pub async fn insert_server_bans(&self, server_bans: &ServerBans) {
        let mut inner = self.inner.write().await;
        for user in &server_bans.users {
            inner.users.insert(user.id.clone(), user.clone());
        }
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
        self.inner
            .write()
            .await
            .servers
            .insert(server.id.clone(), server)
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
        self.inner.write().await.remove_server(id.as_ref())
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
        self.inner
            .write()
            .await
            .remove_role(server_id.as_ref(), role_id.as_ref())
    }

    /// Insert or replace a channel in the cache.
    pub async fn insert_channel(&self, channel: Channel) -> Option<Channel> {
        self.inner.write().await.insert_channel(channel)
    }

    /// Insert or replace several channels.
    pub async fn insert_channels(&self, channels: impl IntoIterator<Item = Channel>) {
        let mut inner = self.inner.write().await;
        for channel in channels {
            inner.insert_channel(channel);
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

    /// Remove a cached channel by ID and evict its cached messages.
    pub async fn remove_channel(&self, id: impl AsRef<str>) -> Option<Channel> {
        self.inner.write().await.remove_channel(id.as_ref())
    }

    /// Insert or replace a server member in the cache.
    pub async fn insert_member(&self, member: Member) -> Option<Member> {
        self.inner.write().await.insert_member(member)
    }

    /// Insert or replace several server members.
    pub async fn insert_members(&self, members: impl IntoIterator<Item = Member>) {
        let mut inner = self.inner.write().await;
        for member in members {
            inner.insert_member(member);
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
            .get(server_id.as_ref())
            .and_then(|members| members.get(user_id.as_ref()))
            .cloned()
    }

    /// Fetch all cached members for a server.
    pub async fn members(&self, server_id: impl AsRef<str>) -> Vec<Member> {
        self.inner
            .read()
            .await
            .members
            .get(server_id.as_ref())
            .map(|members| members.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Remove a cached server member by server and user ID.
    pub async fn remove_member(
        &self,
        server_id: impl AsRef<str>,
        user_id: impl AsRef<str>,
    ) -> Option<Member> {
        self.inner
            .write()
            .await
            .remove_member(server_id.as_ref(), user_id.as_ref())
    }

    /// Insert or replace a message in the bounded message cache.
    pub async fn insert_message(&self, message: Message) -> Option<Message> {
        self.inner.write().await.insert_message(message)
    }

    /// Insert or replace several messages in the bounded message cache.
    pub async fn insert_messages(&self, messages: impl IntoIterator<Item = Message>) {
        let mut inner = self.inner.write().await;
        for message in messages {
            inner.insert_message(message);
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
        self.inner.write().await.remove_message(id.as_ref())
    }

    /// Change the maximum number of messages retained by the cache.
    ///
    /// Setting the capacity to `0` disables message caching. Existing entries are evicted
    /// immediately when the capacity is reduced.
    pub async fn set_message_capacity(&self, capacity: usize) {
        let mut inner = self.inner.write().await;
        inner.message_capacity = capacity;
        inner.enforce_message_capacity();
    }

    /// Return the configured message cache capacity.
    pub async fn message_capacity(&self) -> usize {
        self.inner.read().await.message_capacity
    }

    /// Return the number of cached values per model type.
    pub async fn counts(&self) -> CacheCounts {
        let inner = self.inner.read().await;
        CacheCounts {
            users: inner.users.len(),
            servers: inner.servers.len(),
            roles: inner
                .servers
                .values()
                .map(|server| server.roles.len())
                .sum(),
            channels: inner.channels.len(),
            members: inner.members.values().map(|members| members.len()).sum(),
            messages: inner.messages.len(),
        }
    }

    /// Clear all cached values while preserving cache capacity settings.
    pub async fn clear(&self) {
        let mut inner = self.inner.write().await;
        let message_capacity = inner.message_capacity;
        *inner = CacheInner::with_message_capacity(message_capacity);
    }
}

fn channel_server_id(channel: &Channel) -> Option<&str> {
    match channel {
        Channel::TextChannel(channel) => Some(channel.server.as_str()),
        Channel::VoiceChannel(channel) => Some(channel.server.as_str()),
        _ => None,
    }
}

fn decode_ready<T: DeserializeOwned>(value: &Value, kind: &str) -> Option<T> {
    match from_value(value.clone()) {
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
    let Ok(mut value) = to_value(&*current) else {
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

    match from_value(value) {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Cache;
    use crate::models::{
        Category, Channel, ChannelDeleteEvent, ChannelUpdateEvent, GatewayEvent, Member, MemberId,
        Message, Server, TextChannel,
    };

    fn message(id: &str) -> Message {
        Message {
            id: id.to_owned(),
            nonuce: None,
            channel: "channel".to_owned(),
            author: "user".to_owned(),
            content: id.to_owned(),
            attachments: Vec::new(),
            embeds: None,
            mentions: Vec::new(),
            replies: Vec::new(),
        }
    }

    fn text_channel(id: &str, server_id: &str) -> Channel {
        Channel::TextChannel(TextChannel {
            id: id.to_owned(),
            server: server_id.to_owned(),
            name: "private".to_owned(),
            description: None,
            icon: None,
            last_message_id: None,
            nsfw: false,
            extra: json!({
                "default_permissions": {"a": 0, "d": 1},
                "role_permissions": {"role": {"a": 1, "d": 0}}
            }),
        })
    }

    fn member(server_id: &str, user_id: &str) -> Member {
        Member {
            id: MemberId {
                server: server_id.to_owned(),
                user: user_id.to_owned(),
            },
            ..Member::default()
        }
    }

    #[tokio::test]
    async fn members_are_partitioned_by_server() {
        let cache = Cache::new();
        cache.insert_member(member("one", "user")).await;
        cache.insert_member(member("two", "user")).await;

        assert_eq!(cache.members("one").await.len(), 1);
        assert_eq!(cache.members("two").await.len(), 1);
        assert!(cache.member("one", "user").await.is_some());
        assert!(cache.member("two", "user").await.is_some());
        assert_eq!(cache.counts().await.members, 2);

        cache.remove_server("one").await;
        assert!(cache.member("one", "user").await.is_none());
        assert!(cache.member("two", "user").await.is_some());
        assert_eq!(cache.counts().await.members, 1);
    }

    #[tokio::test]
    async fn message_cache_enforces_capacity() {
        let cache = Cache::new();
        cache.set_message_capacity(2).await;
        cache.insert_message(message("one")).await;
        cache.insert_message(message("two")).await;
        cache.insert_message(message("three")).await;

        assert!(cache.message("one").await.is_none());
        assert!(cache.message("two").await.is_some());
        assert!(cache.message("three").await.is_some());
        assert_eq!(cache.counts().await.messages, 2);
    }

    #[tokio::test]
    async fn clear_preserves_message_capacity() {
        let cache = Cache::new();
        cache.set_message_capacity(3).await;
        cache.insert_message(message("one")).await;
        cache.clear().await;

        assert_eq!(cache.message_capacity().await, 3);
        assert_eq!(cache.counts().await.messages, 0);
    }

    #[tokio::test]
    async fn channel_cache_keeps_server_relationships_coherent() {
        let cache = Cache::new();
        let mut server = Server::default();
        server.id = "server".to_owned();
        server.categories.push(Category {
            id: "category".to_owned(),
            title: "Private".to_owned(),
            channels: vec!["channel".to_owned()],
        });
        cache.insert_server(server).await;

        cache
            .update_from_event(&GatewayEvent::ChannelCreate(text_channel(
                "channel", "server",
            )))
            .await;
        let server = cache
            .server("server")
            .await
            .expect("server should be cached");
        assert_eq!(server.channels, vec!["channel".to_owned()]);

        cache
            .update_from_event(&GatewayEvent::ChannelDelete(ChannelDeleteEvent {
                id: "channel".to_owned(),
            }))
            .await;

        let server = cache
            .server("server")
            .await
            .expect("server should remain cached");
        assert!(server.channels.is_empty());
        assert!(server.categories[0].channels.is_empty());
        assert!(cache.channel("channel").await.is_none());
    }

    #[tokio::test]
    async fn channel_update_merges_permission_overrides() {
        let cache = Cache::new();
        cache
            .insert_channel(text_channel("channel", "server"))
            .await;

        cache
            .update_from_event(&GatewayEvent::ChannelUpdate(ChannelUpdateEvent {
                id: "channel".to_owned(),
                data: json!({
                    "default_permissions": {"a": 1, "d": 0},
                    "role_permissions": {"role": {"a": 3, "d": 0}}
                }),
                clear: Vec::new(),
            }))
            .await;

        let Channel::TextChannel(channel) = cache
            .channel("channel")
            .await
            .expect("channel should remain cached")
        else {
            panic!("expected a text channel");
        };

        assert_eq!(channel.extra["default_permissions"]["a"], 1);
        assert_eq!(channel.extra["role_permissions"]["role"]["a"], 3);
    }
}
