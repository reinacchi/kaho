use crate::{
    error::{KahoError, KahoResult},
    gateway::{GatewayClient, GatewayConfig},
    http::{HttpClient, HttpConfig},
};

#[cfg(feature = "cache")]
use crate::{
    cache::Cache,
    gateway::GatewayEventStream,
    models::{
        Channel, ChannelCloseQuery, ChannelCreate, ChannelUpdate, FetchMembersQuery,
        FetchMessageQuery, GatewayEvent, GroupCreate, Member, MemberList, MemberUpdate, Message,
        MessageEdit, MessageSearch, MessageSend, Role, RoleCreate, RoleCreateResponse,
        RoleRanksUpdate, RoleUpdate, SendFriendRequest, Server, ServerBans, ServerCreate,
        ServerEdit, User, UserUpdate,
    },
};

/// Gateway event stream that keeps the client's cache up to date before
/// yielding each event to the caller.
#[cfg(feature = "cache")]
#[derive(Clone, Debug)]
pub struct CachedGatewayEventStream {
    inner: GatewayEventStream,
}

#[cfg(feature = "cache")]
impl CachedGatewayEventStream {
    /// Wait for the next gateway event.
    ///
    /// The gateway receive pipeline has already synchronized the cache before an event reaches
    /// this stream, so a slow application consumer cannot make cache state lag behind Stoat.
    pub async fn next(&mut self) -> Option<KahoResult<GatewayEvent>> {
        self.inner.next().await
    }
}

/// Represents a kaho client value used by the Stoat API models and endpoints.
#[derive(Clone, Debug)]
pub struct KahoClient {
    /// The http value associated with this kaho client.
    pub http: HttpClient,
    /// The gateway value associated with this kaho client.
    pub gateway: GatewayClient,
    /// The cache value associated with this kaho client.
    #[cfg(feature = "cache")]
    pub cache: Cache,
}

impl KahoClient {
    /// Create a new instance.
    pub fn new(http: HttpClient, mut gateway: GatewayClient) -> Self {
        #[cfg(feature = "cache")]
        let cache = {
            let cache = Cache::new();
            gateway.set_cache(cache.clone());
            cache
        };

        KahoClient {
            http,
            gateway,
            #[cfg(feature = "cache")]
            cache,
        }
    }

    /// Connect the bot to the gateway.
    pub async fn connect(&self) -> KahoResult<()> {
        self.gateway.connect().await
    }

    /// Return a gateway event stream that updates the cache as events arrive.
    #[cfg(feature = "cache")]
    pub fn events(&self) -> CachedGatewayEventStream {
        CachedGatewayEventStream {
            inner: self.gateway.events(),
        }
    }
}

/// Represents a builder pattern for constructing a KahoClient.
#[derive(Clone, Debug)]
pub struct KahoClientBuilder {
    token: Option<String>,
}

impl Default for KahoClientBuilder {
    fn default() -> Self {
        Self { token: None }
    }
}

impl KahoClientBuilder {
    /// Create a new instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the authentication token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Build the client.
    pub fn build(self) -> KahoResult<KahoClient> {
        let token = self
            .token
            .ok_or_else(|| KahoError::Other("Token must be provided".into()))?;

        let http_config = HttpConfig::new(&token)?;
        let gateway_config = GatewayConfig::new(&token)?;

        let http = HttpClient::new(http_config)?;
        let gateway = GatewayClient::new(gateway_config);
        Ok(KahoClient::new(http, gateway))
    }
}

#[cfg(feature = "cache")]
impl KahoClient {
    /// Fetch the current bot user and store it in the cache.
    pub async fn fetch_self_cached(&self) -> KahoResult<User> {
        let user = self.http.fetch_self().await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Fetch a user from the cache, falling back to HTTP when missing.
    pub async fn user(&self, user_id: &str) -> KahoResult<User> {
        if let Some(user) = self.cache.user(user_id).await {
            return Ok(user);
        }

        let user = self.http.fetch_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Fetch a fresh user over HTTP and replace the cached value.
    pub async fn fetch_user_cached(&self, user_id: &str) -> KahoResult<User> {
        let user = self.http.fetch_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Edit a user and replace the cached value with the response.
    pub async fn edit_user_cached(
        &self,
        user_id: &str,
        payload: impl Into<UserUpdate>,
    ) -> KahoResult<User> {
        let user = self.http.edit_user(user_id, payload).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Fetch a server from the cache, falling back to HTTP when missing.
    pub async fn server(&self, server_id: &str) -> KahoResult<Server> {
        if let Some(server) = self.cache.server(server_id).await {
            return Ok(server);
        }

        let server = self.http.fetch_server(server_id).await?;
        self.cache.insert_server(server.clone()).await;
        Ok(server)
    }

    /// Fetch a fresh server over HTTP and replace the cached value.
    pub async fn fetch_server_cached(&self, server_id: &str) -> KahoResult<Server> {
        let server = self.http.fetch_server(server_id).await?;
        self.cache.insert_server(server.clone()).await;
        Ok(server)
    }

    /// Create a server and cache the response.
    pub async fn create_server_cached(
        &self,
        payload: impl Into<ServerCreate>,
    ) -> KahoResult<Server> {
        let server = self.http.create_server(payload).await?;
        self.cache.insert_server(server.clone()).await;
        Ok(server)
    }

    /// Edit a server and replace the cached value with the response.
    pub async fn edit_server_cached(
        &self,
        server_id: &str,
        payload: impl Into<ServerEdit>,
    ) -> KahoResult<Server> {
        let server = self.http.edit_server(server_id, payload).await?;
        self.cache.insert_server(server.clone()).await;
        Ok(server)
    }

    /// Delete a server and evict it from the cache.
    pub async fn delete_server_cached(&self, server_id: &str) -> KahoResult {
        self.http.delete_server(server_id).await?;
        self.cache.remove_server(server_id).await;
        Ok(())
    }

    /// Fetch a role from a cached server, falling back to HTTP when missing.
    pub async fn server_role(&self, server_id: &str, role_id: &str) -> KahoResult<Role> {
        if let Some(role) = self.cache.role(server_id, role_id).await {
            return Ok(role);
        }

        let mut role = self.http.fetch_server_role(server_id, role_id).await?;
        if role.id.is_empty() {
            role.id = role_id.to_owned();
        }
        self.cache
            .insert_role(server_id, role_id.to_owned(), role.clone())
            .await;
        Ok(role)
    }

    /// Fetch a fresh role over HTTP and update an already cached server.
    pub async fn fetch_server_role_cached(
        &self,
        server_id: &str,
        role_id: &str,
    ) -> KahoResult<Role> {
        let mut role = self.http.fetch_server_role(server_id, role_id).await?;
        if role.id.is_empty() {
            role.id = role_id.to_owned();
        }
        self.cache
            .insert_role(server_id, role_id.to_owned(), role.clone())
            .await;
        Ok(role)
    }

    /// Create a role and update an already cached server.
    pub async fn create_server_role_cached(
        &self,
        server_id: &str,
        payload: impl Into<RoleCreate>,
    ) -> KahoResult<RoleCreateResponse> {
        let mut response = self.http.create_server_role(server_id, payload).await?;
        if response.role.id.is_empty() {
            response.role.id = response.id.clone();
        }
        self.cache
            .insert_role(server_id, response.id.clone(), response.role.clone())
            .await;
        Ok(response)
    }

    /// Edit a role and update an already cached server.
    pub async fn edit_server_role_cached(
        &self,
        server_id: &str,
        role_id: &str,
        payload: impl Into<RoleUpdate>,
    ) -> KahoResult<Role> {
        let mut role = self
            .http
            .edit_server_role(server_id, role_id, payload)
            .await?;
        if role.id.is_empty() {
            role.id = role_id.to_owned();
        }
        self.cache
            .insert_role(server_id, role_id.to_owned(), role.clone())
            .await;
        Ok(role)
    }

    /// Delete a role and remove it from the server/member cache immediately.
    pub async fn delete_server_role_cached(&self, server_id: &str, role_id: &str) -> KahoResult {
        self.http.delete_server_role(server_id, role_id).await?;
        self.cache.remove_role(server_id, role_id).await;
        Ok(())
    }

    /// Reorder server roles and update cached role ranks immediately.
    pub async fn set_server_role_ranks_cached(
        &self,
        server_id: &str,
        payload: impl Into<RoleRanksUpdate>,
    ) -> KahoResult {
        let payload = payload.into();
        let ranks = payload.ranks.clone();
        self.http.set_server_role_ranks(server_id, payload).await?;
        self.cache.set_role_ranks(server_id, &ranks).await;
        Ok(())
    }

    /// Fetch a channel from the cache, falling back to HTTP when missing.
    pub async fn channel(&self, channel_id: &str) -> KahoResult<Channel> {
        if let Some(channel) = self.cache.channel(channel_id).await {
            return Ok(channel);
        }

        let channel = self.http.fetch_channel(channel_id).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Fetch a fresh channel over HTTP and replace the cached value.
    pub async fn fetch_channel_cached(&self, channel_id: &str) -> KahoResult<Channel> {
        let channel = self.http.fetch_channel(channel_id).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Fetch direct message channels and cache every returned channel.
    pub async fn fetch_direct_message_channels_cached(&self) -> KahoResult<Vec<Channel>> {
        let channels = self.http.fetch_direct_message_channels().await?;
        self.cache.insert_channels(channels.clone()).await;
        Ok(channels)
    }

    /// Open a direct message channel and cache it.
    pub async fn open_direct_message_cached(&self, user_id: &str) -> KahoResult<Channel> {
        let channel = self.http.open_direct_message(user_id).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Edit a channel and replace the cached value with the response.
    pub async fn edit_channel_cached(
        &self,
        channel_id: &str,
        payload: impl Into<ChannelUpdate>,
    ) -> KahoResult<Channel> {
        let channel = self.http.edit_channel(channel_id, payload).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Close, leave, or delete a channel and evict it from the cache.
    pub async fn close_channel_cached(
        &self,
        channel_id: &str,
        query: impl Into<Option<ChannelCloseQuery>>,
    ) -> KahoResult {
        self.http.close_channel(channel_id, query).await?;
        self.cache.remove_channel(channel_id).await;
        Ok(())
    }

    /// Create a group and cache the returned channel.
    pub async fn create_group_cached(
        &self,
        payload: impl Into<GroupCreate>,
    ) -> KahoResult<Channel> {
        let channel = self.http.create_group(payload).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Create a server channel and cache the returned channel.
    pub async fn create_server_channel_cached(
        &self,
        server_id: &str,
        payload: impl Into<ChannelCreate>,
    ) -> KahoResult<Channel> {
        let channel = self.http.create_server_channel(server_id, payload).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Fetch a message from the cache, falling back to HTTP when missing.
    pub async fn message(&self, channel_id: &str, message_id: &str) -> KahoResult<Message> {
        if let Some(message) = self.cache.message(message_id).await {
            return Ok(message);
        }

        let message = self.http.fetch_message(channel_id, message_id).await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Fetch a fresh message over HTTP and replace the cached value.
    pub async fn fetch_message_cached(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> KahoResult<Message> {
        let message = self.http.fetch_message(channel_id, message_id).await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Fetch messages and cache every returned message.
    pub async fn fetch_messages_cached(
        &self,
        channel_id: &str,
        query: impl Into<Option<FetchMessageQuery>>,
    ) -> KahoResult<Vec<Message>> {
        let messages = self.http.fetch_messages(channel_id, query).await?;
        self.cache.insert_messages(messages.clone()).await;
        Ok(messages)
    }

    /// Send a message and cache the response.
    pub async fn send_message_cached(
        &self,
        channel_id: &str,
        payload: impl Into<MessageSend>,
    ) -> KahoResult<Message> {
        let message = self.http.send_message(channel_id, payload).await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Search messages and cache every returned message.
    pub async fn search_messages_cached(
        &self,
        channel_id: &str,
        payload: impl Into<MessageSearch>,
    ) -> KahoResult<Vec<Message>> {
        let messages = self.http.search_messages(channel_id, payload).await?;
        self.cache.insert_messages(messages.clone()).await;
        Ok(messages)
    }

    /// Edit a message and replace the cached value with the response.
    pub async fn edit_message_cached(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<MessageEdit>,
    ) -> KahoResult<Message> {
        let message = self
            .http
            .edit_message(channel_id, message_id, payload)
            .await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Reply to a message and cache the response.
    pub async fn reply_message_cached(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<MessageSend>,
        mention: bool,
    ) -> KahoResult<Message> {
        let message = self
            .http
            .reply_message(channel_id, message_id, payload, mention)
            .await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Delete a message and evict it from the cache.
    pub async fn delete_message_cached(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.http.delete_message(channel_id, message_id).await?;
        self.cache.remove_message(message_id).await;
        Ok(())
    }

    /// Bulk-delete messages and evict all deleted message IDs from the cache.
    pub async fn bulk_delete_messages_cached(
        &self,
        channel_id: &str,
        message_ids: Vec<String>,
    ) -> KahoResult {
        self.http
            .bulk_delete_messages(channel_id, message_ids.clone())
            .await?;
        for message_id in message_ids {
            self.cache.remove_message(message_id).await;
        }
        Ok(())
    }

    /// Fetch group members and cache every returned user.
    pub async fn fetch_group_members_cached(&self, channel_id: &str) -> KahoResult<Vec<User>> {
        let users = self.http.fetch_group_members(channel_id).await?;
        self.cache.insert_users(users.clone()).await;
        Ok(users)
    }

    /// Fetch a server member from the cache, falling back to HTTP when missing.
    pub async fn server_member(&self, server_id: &str, member_id: &str) -> KahoResult<Member> {
        if let Some(member) = self.cache.member(server_id, member_id).await {
            return Ok(member);
        }

        let member = self.http.fetch_server_member(server_id, member_id).await?;
        self.cache.insert_member(member.clone()).await;
        Ok(member)
    }

    /// Fetch a fresh server member over HTTP and replace the cached value.
    pub async fn fetch_server_member_cached(
        &self,
        server_id: &str,
        member_id: &str,
    ) -> KahoResult<Member> {
        let member = self.http.fetch_server_member(server_id, member_id).await?;
        self.cache.insert_member(member.clone()).await;
        Ok(member)
    }

    /// Edit a server member and replace the cached value with the response.
    pub async fn edit_server_member_cached(
        &self,
        server_id: &str,
        member_id: &str,
        payload: impl Into<MemberUpdate>,
    ) -> KahoResult<Member> {
        let member = self
            .http
            .edit_server_member(server_id, member_id, payload)
            .await?;
        self.cache.insert_member(member.clone()).await;
        Ok(member)
    }

    /// Kick a server member and evict their cached member state.
    pub async fn kick_server_member_cached(&self, server_id: &str, member_id: &str) -> KahoResult {
        self.http.kick_server_member(server_id, member_id).await?;
        self.cache.remove_member(server_id, member_id).await;
        Ok(())
    }

    /// Fetch server members and cache every returned user and member.
    pub async fn fetch_server_members_cached(
        &self,
        server_id: &str,
        query: impl Into<Option<FetchMembersQuery>>,
    ) -> KahoResult<MemberList> {
        let member_list = self.http.fetch_server_members(server_id, query).await?;
        self.cache.insert_member_list(&member_list).await;
        Ok(member_list)
    }

    /// Fetch server bans and cache every returned user.
    pub async fn fetch_server_bans_cached(&self, server_id: &str) -> KahoResult<ServerBans> {
        let bans = self.http.fetch_server_bans(server_id).await?;
        self.cache.insert_server_bans(&bans).await;
        Ok(bans)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn accept_friend_request_cached(&self, user_id: &str) -> KahoResult<User> {
        let user = self.http.accept_friend_request(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn remove_friend_cached(&self, user_id: &str) -> KahoResult<User> {
        let user = self.http.remove_friend(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn block_user_cached(&self, user_id: &str) -> KahoResult<User> {
        let user = self.http.block_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn unblock_user_cached(&self, user_id: &str) -> KahoResult<User> {
        let user = self.http.unblock_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Send a friend request and cache the returned user.
    pub async fn send_friend_request_cached(
        &self,
        payload: impl Into<SendFriendRequest>,
    ) -> KahoResult<User> {
        let user = self.http.send_friend_request(payload).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }
}
