use {
    reqwest::{
        header::{HeaderMap, HeaderValue},
        Client,
    },
    serde::{de::DeserializeOwned, ser::Serialize},
};

use crate::{
    error::{KahoError, KahoResult},
    http::{endpoint::Endpoint, HttpConfig},
    models::{
        FetchMessageQuery, FlagResponse, Message, MessageEdit, MessageReplyIntent, MessageSearch,
        MessageSend, PublicBot, User, UserUpdate,
    },
};

/// HTTP client for calling the Stoat REST API.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: HttpConfig,
}

impl HttpClient {
    /// Create an HTTP client from configuration.
    pub fn new(config: HttpConfig) -> KahoResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("X-Bot-Token", HeaderValue::from_str(&config.token)?);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client, config })
    }

    fn make_url(&self, path: impl AsRef<str>) -> String {
        format!(
            "{}/{}",
            self.config.api_url.trim_end_matches('/'),
            path.as_ref().trim_start_matches('/')
        )
    }

    /// Send a GET request and deserialize the JSON response.
    pub async fn get<T: DeserializeOwned>(&self, path: impl AsRef<str>) -> KahoResult<T> {
        let response = self.client.get(self.make_url(path)).send().await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    /// Send a POST request with a JSON payload and deserialize the JSON response.
    pub async fn post<T: DeserializeOwned, U: Serialize>(
        &self,
        path: impl AsRef<str>,
        payload: U,
    ) -> KahoResult<T> {
        let response = self
            .client
            .post(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    /// Send a PUT request with a JSON payload.
    pub async fn put<T: Serialize>(&self, path: impl AsRef<str>, payload: T) -> KahoResult {
        let response = self
            .client
            .put(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        Ok(())
    }

    /// Send a PATCH request with a JSON payload and deserialize the JSON response.
    pub async fn patch<T: Serialize, R: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        payload: T,
    ) -> KahoResult<R> {
        let response = self
            .client
            .patch(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    /// Send a DELETE request.
    pub async fn delete<T: Serialize>(
        &self,
        path: impl AsRef<str>,
        payload: Option<T>,
    ) -> KahoResult {
        let mut request = self.client.delete(self.make_url(path));

        if let Some(payload) = payload {
            request = request.json(&payload);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        Ok(())
    }

    // Bot-related methods
    /// Get a public bot.
    pub async fn fetch_public_bot(&self, bot_id: &str) -> KahoResult<PublicBot> {
        self.get(Endpoint::BotInvite(bot_id.to_string()).path())
            .await
    }

    // User-related methods
    /// Get properties of the bot user.
    pub async fn fetch_self(&self) -> KahoResult<User> {
        self.get(Endpoint::User("@me".to_string()).path()).await
    }

    /// Edit a user.
    pub async fn edit_user(
        &self,
        user_id: &str,
        payload: impl Into<UserUpdate>,
    ) -> KahoResult<User> {
        self.patch(Endpoint::User(user_id.to_string()).path(), payload.into())
            .await
    }

    /// Get properties of the targeted user.
    pub async fn fetch_user(&self, user_id: &str) -> KahoResult<User> {
        self.get(Endpoint::User(user_id.to_string()).path()).await
    }

    /// Get the flags of the targeted user.
    pub async fn fetch_user_flags(&self, user_id: &str) -> KahoResult<FlagResponse> {
        self.get(Endpoint::UserFlags(user_id.to_string()).path())
            .await
    }

    // Message-related methods
    /// Acknowledge a message in the specified channel.
    pub async fn acknowledge_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.put(
            Endpoint::ChannelMessageAck(channel_id.to_string(), message_id.to_string()).path(),
            (),
        )
        .await
    }

    /// Fetch messages in the specified channel.
    pub async fn fetch_messages(
        &self,
        channel_id: &str,
        query: impl Into<Option<FetchMessageQuery>>,
    ) -> KahoResult<Vec<Message>> {
        let mut path = Endpoint::ChannelMessages(channel_id.to_string()).path();

        if let Some(q) = query.into() {
            let encoded_query = serde_urlencoded::to_string(&q).unwrap();
            path.push('?');
            path.push_str(&encoded_query);
        }

        self.get(path).await
    }

    /// Send a message in the specified channel.
    pub async fn send_message(
        &self,
        channel_id: &str,
        payload: impl Into<MessageSend>,
    ) -> KahoResult<Message> {
        self.post(
            Endpoint::ChannelMessages(channel_id.to_string()).path(),
            payload.into(),
        )
        .await
    }

    /// Search for messages in the specified channel matching the query.
    pub async fn search_messages(
        &self,
        channel_id: &str,
        payload: impl Into<MessageSearch>,
    ) -> KahoResult<Vec<Message>> {
        self.post(
            Endpoint::ChannelMessageSearch(channel_id.to_string()).path(),
            payload.into(),
        )
        .await
    }

    /// Pin a message in the specified channel.
    pub async fn pin_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.post(
            Endpoint::ChannelMessagePin(channel_id.to_string(), message_id.to_string()).path(),
            (),
        )
        .await
    }

    /// Unpin a message in the specified channel.
    pub async fn unpin_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ChannelMessagePin(channel_id.to_string(), message_id.to_string()).path(),
            None::<()>,
        )
        .await
    }

    /// Fetch a message in the specified channel.
    pub async fn fetch_message(&self, channel_id: &str, message_id: &str) -> KahoResult<Message> {
        self.get(Endpoint::ChannelMessage(channel_id.to_string(), message_id.to_string()).path())
            .await
    }

    /// Delete a message in the specified channel.
    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ChannelMessage(channel_id.to_string(), message_id.to_string()).path(),
            None::<()>,
        )
        .await
    }

    /// Edit a message in the specified channel.
    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<MessageEdit>,
    ) -> KahoResult<Message> {
        self.patch(
            Endpoint::ChannelMessage(channel_id.to_string(), message_id.to_string()).path(),
            payload.into(),
        )
        .await
    }

    /// Bulk delete messages in the specified channel.
    pub async fn bulk_delete_messages(
        &self,
        channel_id: &str,
        message_ids: Vec<String>,
    ) -> KahoResult {
        let payload = serde_json::json!({ "ids": message_ids });

        self.post(
            Endpoint::ChannelMessageBulk(channel_id.to_string()).path(),
            payload,
        )
        .await
    }

    /// Reply to a certain message in the specified channel.
    pub async fn reply_message(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<MessageSend>,
        mention: bool,
    ) -> KahoResult<Message> {
        let reply_intent = MessageReplyIntent {
            id: message_id.to_string().into(),
            mention,
            fail_if_not_exists: true,
        };

        let mut message_payload: MessageSend = payload.into();
        message_payload.replies.push(reply_intent);

        self.send_message(channel_id, message_payload).await
    }
}
