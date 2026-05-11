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
        Account, AccountChangeEmail, AccountChangePassword, AccountCreate,
        AccountPasswordConfirmation, AccountPasswordReset, AccountResendVerification,
        AccountSendPasswordReset, Bot, BotCreate, BotCreateResponse, BotInvite, BotInviteResponse,
        BotUpdate, ChangeUsername, DefaultAvatar, Emoji, EmojiCreate, FetchMessageQuery,
        FlagResponse, Invite, InviteJoinResponse, Message, MessageEdit, MessageReplyIntent,
        MessageSearch, MessageSend, MutualResponse, PublicBot, SendFriendRequest, User,
        UserProfile, UserUpdate,
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

    /// Send a GET request and return the raw response bytes.
    pub async fn get_bytes(&self, path: impl AsRef<str>) -> KahoResult<Vec<u8>> {
        let response = self.client.get(self.make_url(path)).send().await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        Ok(response.bytes().await?.to_vec())
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

    /// Send a POST request with a JSON payload and ignore the response body.
    pub async fn post_empty<U: Serialize>(&self, path: impl AsRef<str>, payload: U) -> KahoResult {
        let response = self
            .client
            .post(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        Ok(())
    }

    /// Send a PATCH request with a JSON payload and ignore the response body.
    pub async fn patch_empty<U: Serialize>(&self, path: impl AsRef<str>, payload: U) -> KahoResult {
        let response = self
            .client
            .patch(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        Ok(())
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

    /// Send a PUT request with a JSON payload and deserialize the JSON response.
    pub async fn put_return<T: Serialize, R: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        payload: T,
    ) -> KahoResult<R> {
        let response = self
            .client
            .put(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
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

    /// Send a DELETE request and deserialize the JSON response.
    pub async fn delete_return<T: Serialize, R: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        payload: Option<T>,
    ) -> KahoResult<R> {
        let mut request = self.client.delete(self.make_url(path));

        if let Some(payload) = payload {
            request = request.json(&payload);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(KahoError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    // Account-related methods
    /// Create a new account.
    pub async fn create_account(&self, payload: impl Into<AccountCreate>) -> KahoResult {
        self.post_empty(Endpoint::AccountCreate.path(), payload.into())
            .await
    }

    /// Resend an account verification email.
    pub async fn resend_account_verification(
        &self,
        payload: impl Into<AccountResendVerification>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountReverify.path(), payload.into())
            .await
    }

    /// Confirm account deletion.
    pub async fn confirm_account_deletion(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.put(Endpoint::AccountDelete.path(), payload.into())
            .await
    }

    /// Request account deletion.
    pub async fn delete_account(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountDelete.path(), payload.into())
            .await
    }

    /// Fetch account information.
    pub async fn fetch_account(&self) -> KahoResult<Account> {
        self.get(Endpoint::Account.path()).await
    }

    /// Disable the current account.
    pub async fn disable_account(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountDisable.path(), payload.into())
            .await
    }

    /// Change the current account password.
    pub async fn change_password(&self, payload: impl Into<AccountChangePassword>) -> KahoResult {
        self.patch_empty(Endpoint::AccountChangePassword.path(), payload.into())
            .await
    }

    /// Change the current account email.
    pub async fn change_email(&self, payload: impl Into<AccountChangeEmail>) -> KahoResult {
        self.patch_empty(Endpoint::AccountChangeEmail.path(), payload.into())
            .await
    }

    /// Verify an account email with a verification code.
    pub async fn verify_email(&self, code: &str) -> KahoResult {
        self.post_empty(Endpoint::AccountVerify(code.to_string()).path(), ())
            .await
    }

    /// Send a password reset email.
    pub async fn send_password_reset(
        &self,
        payload: impl Into<AccountSendPasswordReset>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountResetPassword.path(), payload.into())
            .await
    }

    /// Complete a password reset.
    pub async fn reset_password(&self, payload: impl Into<AccountPasswordReset>) -> KahoResult {
        self.patch_empty(Endpoint::AccountResetPassword.path(), payload.into())
            .await
    }

    // Bot-related methods
    /// Get a public bot.
    pub async fn fetch_public_bot(&self, bot_id: &str) -> KahoResult<PublicBot> {
        self.get(Endpoint::BotInvite(bot_id.to_string()).path())
            .await
    }

    /// Create a new bot.
    pub async fn create_bot(&self, payload: impl Into<BotCreate>) -> KahoResult<BotCreateResponse> {
        self.post(Endpoint::BotCreate().path(), payload.into())
            .await
    }

    /// Invite a public bot to a server.
    pub async fn invite_bot(
        &self,
        bot_id: &str,
        payload: impl Into<BotInvite>,
    ) -> KahoResult<BotInviteResponse> {
        self.post(
            Endpoint::BotInvite(bot_id.to_string()).path(),
            payload.into(),
        )
        .await
    }

    /// Fetch a bot by ID.
    pub async fn fetch_bot(&self, bot_id: &str) -> KahoResult<Bot> {
        self.get(Endpoint::Bot(bot_id.to_string()).path()).await
    }

    /// Delete a bot by ID.
    pub async fn delete_bot(&self, bot_id: &str) -> KahoResult {
        self.delete(Endpoint::Bot(bot_id.to_string()).path(), None::<()>)
            .await
    }

    /// Edit a bot by ID.
    pub async fn edit_bot(&self, bot_id: &str, payload: impl Into<BotUpdate>) -> KahoResult<Bot> {
        self.patch(Endpoint::Bot(bot_id.to_string()).path(), payload.into())
            .await
    }

    /// Fetch bots owned by the current account.
    pub async fn fetch_owned_bots(&self) -> KahoResult<Vec<Bot>> {
        self.get(Endpoint::BotsOwned.path()).await
    }

    // User-related methods
    /// Get properties of the bot user.
    pub async fn fetch_self(&self) -> KahoResult<User> {
        self.get(Endpoint::UserMe.path()).await
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

    /// Change the current user's username.
    pub async fn change_username(&self, payload: impl Into<ChangeUsername>) -> KahoResult<User> {
        self.patch(Endpoint::UserUsername().path(), payload.into())
            .await
    }

    /// Fetch the default avatar for a user.
    pub async fn fetch_default_avatar(&self, user_id: &str) -> KahoResult<DefaultAvatar> {
        let bytes = self
            .get_bytes(Endpoint::UserDefaultAvatar(user_id.to_string()).path())
            .await?;

        Ok(DefaultAvatar { bytes })
    }

    /// Fetch a user's profile.
    pub async fn fetch_user_profile(&self, user_id: &str) -> KahoResult<UserProfile> {
        self.get(Endpoint::UserProfile(user_id.to_string()).path())
            .await
    }

    /// Fetch mutual friends, servers, groups, and DMs for a user.
    pub async fn fetch_mutual_relationships(&self, user_id: &str) -> KahoResult<MutualResponse> {
        self.get(Endpoint::RelationshipMutual(user_id.to_string()).path())
            .await
    }

    /// Accept an incoming friend request.
    pub async fn accept_friend_request(&self, user_id: &str) -> KahoResult<User> {
        self.put_return(Endpoint::RelationshipFriend(user_id.to_string()).path(), ())
            .await
    }

    /// Deny a friend request or remove a friend.
    pub async fn remove_friend(&self, user_id: &str) -> KahoResult<User> {
        self.delete_return(
            Endpoint::RelationshipFriend(user_id.to_string()).path(),
            None::<()>,
        )
        .await
    }

    /// Block a user.
    pub async fn block_user(&self, user_id: &str) -> KahoResult<User> {
        self.put_return(Endpoint::RelationshipBlock(user_id.to_string()).path(), ())
            .await
    }

    /// Unblock a user.
    pub async fn unblock_user(&self, user_id: &str) -> KahoResult<User> {
        self.delete_return(
            Endpoint::RelationshipBlock(user_id.to_string()).path(),
            None::<()>,
        )
        .await
    }

    /// Send a friend request.
    pub async fn send_friend_request(
        &self,
        payload: impl Into<SendFriendRequest>,
    ) -> KahoResult<User> {
        self.post(Endpoint::RelationshipFriends().path(), payload.into())
            .await
    }

    // Custom emoji methods
    /// Fetch a custom emoji.
    pub async fn fetch_emoji(&self, emoji_id: &str) -> KahoResult<Emoji> {
        self.get(Endpoint::Emoji(emoji_id.to_string()).path()).await
    }

    /// Create or replace a custom emoji.
    pub async fn create_emoji(
        &self,
        emoji_id: &str,
        payload: impl Into<EmojiCreate>,
    ) -> KahoResult<Emoji> {
        self.put_return(Endpoint::Emoji(emoji_id.to_string()).path(), payload.into())
            .await
    }

    /// Delete a custom emoji.
    pub async fn delete_emoji(&self, emoji_id: &str) -> KahoResult {
        self.delete(Endpoint::Emoji(emoji_id.to_string()).path(), None::<()>)
            .await
    }

    // Invite methods
    /// Fetch an invite.
    pub async fn fetch_invite(&self, invite_id: &str) -> KahoResult<Invite> {
        self.get(Endpoint::Invite(invite_id.to_string()).path())
            .await
    }

    /// Join or accept an invite.
    pub async fn accept_invite(&self, invite_id: &str) -> KahoResult<InviteJoinResponse> {
        self.post(Endpoint::Invite(invite_id.to_string()).path(), ())
            .await
    }

    /// Delete an invite.
    pub async fn delete_invite(&self, invite_id: &str) -> KahoResult {
        self.delete(Endpoint::Invite(invite_id.to_string()).path(), None::<()>)
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
