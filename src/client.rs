use crate::{
    error::{KahoError, KahoResult},
    gateway::{GatewayClient, GatewayConfig},
    http::{HttpClient, HttpConfig},
};

#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "cache")]
use crate::{gateway::GatewayEventStream, models::GatewayEvent};


/// Gateway event stream that keeps the client's cache up to date before
/// yielding each event to the caller.
#[cfg(feature = "cache")]
#[derive(Clone, Debug)]
pub struct CachedGatewayEventStream {
    inner: GatewayEventStream,
    cache: Cache,
}

#[cfg(feature = "cache")]
impl CachedGatewayEventStream {
    /// Wait for the next gateway event, updating the cache first when possible.
    pub async fn next(&mut self) -> Option<KahoResult<GatewayEvent>> {
        match self.inner.next().await {
            Some(Ok(event)) => {
                self.cache.update_from_event(&event).await;
                Some(Ok(event))
            }
            other => other,
        }
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
    /// Calls the Stoat API or client internals to new for this resource.
    pub fn new(http: HttpClient, gateway: GatewayClient) -> Self {
        KahoClient {
            http,
            gateway,
            #[cfg(feature = "cache")]
            cache: Cache::new(),
        }
    }

    /// Connect the bot to the gateway.
    pub async fn connect(&mut self) -> KahoResult<()> {
        self.gateway.connect().await
    }

    /// Return a gateway event stream that updates the cache as events arrive.
    #[cfg(feature = "cache")]
    pub fn events(&self) -> CachedGatewayEventStream {
        CachedGatewayEventStream {
            inner: self.gateway.events(),
            cache: self.cache.clone(),
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
    /// Calls the Stoat API or client internals to new for this resource.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calls the Stoat API or client internals to token for this resource.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Calls the Stoat API or client internals to build for this resource.
    pub fn build(self) -> KahoResult<KahoClient> {
        let token = self
            .token
            .ok_or_else(|| KahoError::Other("Token must be provided".into()))?;

        let http_config = HttpConfig::new(&token)?;
        let gateway_config = GatewayConfig::new(&token)?;

        Ok(KahoClient {
            http: HttpClient::new(http_config)?,
            gateway: GatewayClient::new(gateway_config),
            #[cfg(feature = "cache")]
            cache: Cache::new(),
        })
    }
}

#[cfg(feature = "cache")]
impl KahoClient {
    /// Fetch the current bot user and store it in the cache.
    pub async fn fetch_self_cached(&self) -> KahoResult<crate::models::User> {
        let user = self.http.fetch_self().await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Fetch a user from the cache, falling back to HTTP when missing.
    pub async fn user(&self, user_id: &str) -> KahoResult<crate::models::User> {
        if let Some(user) = self.cache.user(user_id).await {
            return Ok(user);
        }

        let user = self.http.fetch_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Fetch a fresh user over HTTP and replace the cached value.
    pub async fn fetch_user_cached(&self, user_id: &str) -> KahoResult<crate::models::User> {
        let user = self.http.fetch_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Edit a user and replace the cached value with the response.
    pub async fn edit_user_cached(
        &self,
        user_id: &str,
        payload: impl Into<crate::models::UserUpdate>,
    ) -> KahoResult<crate::models::User> {
        let user = self.http.edit_user(user_id, payload).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Fetch a server from the cache, falling back to HTTP when missing.
    pub async fn server(&self, server_id: &str) -> KahoResult<crate::models::Server> {
        if let Some(server) = self.cache.server(server_id).await {
            return Ok(server);
        }

        let server = self.http.fetch_server(server_id).await?;
        self.cache.insert_server(server.clone()).await;
        Ok(server)
    }

    /// Fetch a fresh server over HTTP and replace the cached value.
    pub async fn fetch_server_cached(&self, server_id: &str) -> KahoResult<crate::models::Server> {
        let server = self.http.fetch_server(server_id).await?;
        self.cache.insert_server(server.clone()).await;
        Ok(server)
    }

    /// Create a server and cache the response.
    pub async fn create_server_cached(
        &self,
        payload: impl Into<crate::models::ServerCreate>,
    ) -> KahoResult<crate::models::Server> {
        let server = self.http.create_server(payload).await?;
        self.cache.insert_server(server.clone()).await;
        Ok(server)
    }

    /// Edit a server and replace the cached value with the response.
    pub async fn edit_server_cached(
        &self,
        server_id: &str,
        payload: impl Into<crate::models::ServerEdit>,
    ) -> KahoResult<crate::models::Server> {
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

    /// Fetch a channel from the cache, falling back to HTTP when missing.
    pub async fn channel(&self, channel_id: &str) -> KahoResult<crate::models::Channel> {
        if let Some(channel) = self.cache.channel(channel_id).await {
            return Ok(channel);
        }

        let channel = self.http.fetch_channel(channel_id).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Fetch a fresh channel over HTTP and replace the cached value.
    pub async fn fetch_channel_cached(&self, channel_id: &str) -> KahoResult<crate::models::Channel> {
        let channel = self.http.fetch_channel(channel_id).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Fetch direct message channels and cache every returned channel.
    pub async fn fetch_direct_message_channels_cached(&self) -> KahoResult<Vec<crate::models::Channel>> {
        let channels = self.http.fetch_direct_message_channels().await?;
        self.cache.insert_channels(channels.clone()).await;
        Ok(channels)
    }

    /// Open a direct message channel and cache it.
    pub async fn open_direct_message_cached(&self, user_id: &str) -> KahoResult<crate::models::Channel> {
        let channel = self.http.open_direct_message(user_id).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Edit a channel and replace the cached value with the response.
    pub async fn edit_channel_cached(
        &self,
        channel_id: &str,
        payload: impl Into<crate::models::ChannelUpdate>,
    ) -> KahoResult<crate::models::Channel> {
        let channel = self.http.edit_channel(channel_id, payload).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Close, leave, or delete a channel and evict it from the cache.
    pub async fn close_channel_cached(
        &self,
        channel_id: &str,
        query: impl Into<Option<crate::models::ChannelCloseQuery>>,
    ) -> KahoResult {
        self.http.close_channel(channel_id, query).await?;
        self.cache.remove_channel(channel_id).await;
        Ok(())
    }

    /// Create a group and cache the returned channel.
    pub async fn create_group_cached(
        &self,
        payload: impl Into<crate::models::GroupCreate>,
    ) -> KahoResult<crate::models::Channel> {
        let channel = self.http.create_group(payload).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Create a server channel and cache the returned channel.
    pub async fn create_server_channel_cached(
        &self,
        server_id: &str,
        payload: impl Into<crate::models::ChannelCreate>,
    ) -> KahoResult<crate::models::Channel> {
        let channel = self.http.create_server_channel(server_id, payload).await?;
        self.cache.insert_channel(channel.clone()).await;
        Ok(channel)
    }

    /// Fetch a message from the cache, falling back to HTTP when missing.
    pub async fn message(&self, channel_id: &str, message_id: &str) -> KahoResult<crate::models::Message> {
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
    ) -> KahoResult<crate::models::Message> {
        let message = self.http.fetch_message(channel_id, message_id).await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Fetch messages and cache every returned message.
    pub async fn fetch_messages_cached(
        &self,
        channel_id: &str,
        query: impl Into<Option<crate::models::FetchMessageQuery>>,
    ) -> KahoResult<Vec<crate::models::Message>> {
        let messages = self.http.fetch_messages(channel_id, query).await?;
        self.cache.insert_messages(messages.clone()).await;
        Ok(messages)
    }

    /// Send a message and cache the response.
    pub async fn send_message_cached(
        &self,
        channel_id: &str,
        payload: impl Into<crate::models::MessageSend>,
    ) -> KahoResult<crate::models::Message> {
        let message = self.http.send_message(channel_id, payload).await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Search messages and cache every returned message.
    pub async fn search_messages_cached(
        &self,
        channel_id: &str,
        payload: impl Into<crate::models::MessageSearch>,
    ) -> KahoResult<Vec<crate::models::Message>> {
        let messages = self.http.search_messages(channel_id, payload).await?;
        self.cache.insert_messages(messages.clone()).await;
        Ok(messages)
    }

    /// Edit a message and replace the cached value with the response.
    pub async fn edit_message_cached(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<crate::models::MessageEdit>,
    ) -> KahoResult<crate::models::Message> {
        let message = self.http.edit_message(channel_id, message_id, payload).await?;
        self.cache.insert_message(message.clone()).await;
        Ok(message)
    }

    /// Reply to a message and cache the response.
    pub async fn reply_message_cached(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<crate::models::MessageSend>,
        mention: bool,
    ) -> KahoResult<crate::models::Message> {
        let message = self.http.reply_message(channel_id, message_id, payload, mention).await?;
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
        self.http.bulk_delete_messages(channel_id, message_ids.clone()).await?;
        for message_id in message_ids {
            self.cache.remove_message(message_id).await;
        }
        Ok(())
    }

    /// Fetch group members and cache every returned user.
    pub async fn fetch_group_members_cached(&self, channel_id: &str) -> KahoResult<Vec<crate::models::User>> {
        let users = self.http.fetch_group_members(channel_id).await?;
        self.cache.insert_users(users.clone()).await;
        Ok(users)
    }

    /// Fetch server members and cache every returned user.
    pub async fn fetch_server_members_cached(
        &self,
        server_id: &str,
        query: impl Into<Option<crate::models::FetchMembersQuery>>,
    ) -> KahoResult<crate::models::MemberList> {
        let member_list = self.http.fetch_server_members(server_id, query).await?;
        self.cache.insert_member_list(&member_list).await;
        Ok(member_list)
    }

    /// Fetch server bans and cache every returned user.
    pub async fn fetch_server_bans_cached(&self, server_id: &str) -> KahoResult<crate::models::ServerBans> {
        let bans = self.http.fetch_server_bans(server_id).await?;
        self.cache.insert_server_bans(&bans).await;
        Ok(bans)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn accept_friend_request_cached(&self, user_id: &str) -> KahoResult<crate::models::User> {
        let user = self.http.accept_friend_request(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn remove_friend_cached(&self, user_id: &str) -> KahoResult<crate::models::User> {
        let user = self.http.remove_friend(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn block_user_cached(&self, user_id: &str) -> KahoResult<crate::models::User> {
        let user = self.http.block_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Relationship helpers return updated user models; cache them.
    pub async fn unblock_user_cached(&self, user_id: &str) -> KahoResult<crate::models::User> {
        let user = self.http.unblock_user(user_id).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }

    /// Send a friend request and cache the returned user.
    pub async fn send_friend_request_cached(
        &self,
        payload: impl Into<crate::models::SendFriendRequest>,
    ) -> KahoResult<crate::models::User> {
        let user = self.http.send_friend_request(payload).await?;
        self.cache.insert_user(user.clone()).await;
        Ok(user)
    }
}
