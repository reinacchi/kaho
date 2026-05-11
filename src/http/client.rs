use {
    reqwest::{
        header::{HeaderMap, HeaderValue},
        Client, Method, RequestBuilder, Response, StatusCode,
    },
    serde::{de::DeserializeOwned, ser::Serialize},
};

use crate::{
    error::{KahoError, KahoResult},
    http::{endpoint::Endpoint, HttpConfig, RateLimitedResponse, RateLimiter},
    models::*,
};

/// HTTP client for calling the Stoat REST API.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: HttpConfig,
    rate_limiter: std::sync::Arc<RateLimiter>,
}

impl HttpClient {
    /// Calls the Stoat API or client internals to new for this resource.
    pub fn new(config: HttpConfig) -> KahoResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("X-Bot-Token", HeaderValue::from_str(&config.token)?);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            config,
            rate_limiter: std::sync::Arc::new(RateLimiter::default()),
        })
    }

    fn make_url(&self, path: impl AsRef<str>) -> String {
        format!(
            "{}/{}",
            self.config.api_url.trim_end_matches('/'),
            path.as_ref().trim_start_matches('/')
        )
    }

    async fn send_rate_limited(
        &self,
        method: Method,
        path: &str,
        build_request: impl Fn() -> RequestBuilder,
    ) -> KahoResult<Response> {
        loop {
            self.rate_limiter.wait(&method, path).await;

            let response = build_request().send().await?;
            self.rate_limiter
                .update_from_headers(&method, path, response.headers())
                .await;

            if response.status() != StatusCode::TOO_MANY_REQUESTS {
                if !response.status().is_success() {
                    return Err(KahoError::FailedRequest(response));
                }

                return Ok(response);
            }

            let retry_after = response
                .json::<RateLimitedResponse>()
                .await
                .map(|payload| payload.retry_after)
                .unwrap_or(10_000);

            self.rate_limiter
                .update_retry_after(&method, path, retry_after)
                .await;
            tokio::time::sleep(std::time::Duration::from_millis(retry_after)).await;
        }
    }

    /// Send a GET request and deserialize the JSON response.
    pub async fn get<T: DeserializeOwned>(&self, path: impl AsRef<str>) -> KahoResult<T> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::GET, path, || self.client.get(self.make_url(path)))
            .await?;

        Ok(response.json().await?)
    }

    /// Send a GET request and return the raw response bytes.
    pub async fn get_bytes(&self, path: impl AsRef<str>) -> KahoResult<Vec<u8>> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::GET, path, || self.client.get(self.make_url(path)))
            .await?;

        Ok(response.bytes().await?.to_vec())
    }

    /// Send a POST request with a JSON payload and deserialize the JSON response.
    pub async fn post<T: DeserializeOwned, U: Serialize>(
        &self,
        path: impl AsRef<str>,
        payload: U,
    ) -> KahoResult<T> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::POST, path, || {
                self.client.post(self.make_url(path)).json(&payload)
            })
            .await?;

        Ok(response.json().await?)
    }

    /// Send a POST request with a JSON payload and ignore the response body.
    pub async fn post_empty<U: Serialize>(&self, path: impl AsRef<str>, payload: U) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::POST, path, || {
            self.client.post(self.make_url(path)).json(&payload)
        })
        .await?;

        Ok(())
    }

    /// Send a PATCH request with a JSON payload and ignore the response body.
    pub async fn patch_empty<U: Serialize>(&self, path: impl AsRef<str>, payload: U) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::PATCH, path, || {
            self.client.patch(self.make_url(path)).json(&payload)
        })
        .await?;

        Ok(())
    }

    /// Send a PUT request with a JSON payload.
    pub async fn put<T: Serialize>(&self, path: impl AsRef<str>, payload: T) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::PUT, path, || {
            self.client.put(self.make_url(path)).json(&payload)
        })
        .await?;

        Ok(())
    }

    /// Send a PUT request with a JSON payload and deserialize the JSON response.
    pub async fn put_return<T: Serialize, R: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        payload: T,
    ) -> KahoResult<R> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::PUT, path, || {
                self.client.put(self.make_url(path)).json(&payload)
            })
            .await?;

        Ok(response.json().await?)
    }

    /// Send a PATCH request with a JSON payload and deserialize the JSON response.
    pub async fn patch<T: Serialize, R: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        payload: T,
    ) -> KahoResult<R> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::PATCH, path, || {
                self.client.patch(self.make_url(path)).json(&payload)
            })
            .await?;

        Ok(response.json().await?)
    }

    /// Calls the Stoat API or client internals to delete for this resource.
    pub async fn delete<T: Serialize>(
        &self,
        path: impl AsRef<str>,
        payload: Option<T>,
    ) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::DELETE, path, || {
            let mut request = self.client.delete(self.make_url(path));

            if let Some(payload) = payload.as_ref() {
                request = request.json(payload);
            }

            request
        })
        .await?;

        Ok(())
    }

    /// Send a DELETE request and deserialize the JSON response.
    pub async fn delete_return<T: Serialize, R: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        payload: Option<T>,
    ) -> KahoResult<R> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::DELETE, path, || {
                let mut request = self.client.delete(self.make_url(path));

                if let Some(payload) = payload.as_ref() {
                    request = request.json(payload);
                }

                request
            })
            .await?;

        Ok(response.json().await?)
    }

    // Account-related methods
    /// Calls the Stoat API or client internals to create account for this resource.
    pub async fn create_account(&self, payload: impl Into<AccountCreate>) -> KahoResult {
        self.post_empty(Endpoint::AccountCreate.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to resend account verification for this resource.
    pub async fn resend_account_verification(
        &self,
        payload: impl Into<AccountResendVerification>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountReverify.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to confirm account deletion for this resource.
    pub async fn confirm_account_deletion(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.put(Endpoint::AccountDelete.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to delete account for this resource.
    pub async fn delete_account(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountDelete.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to fetch account for this resource.
    pub async fn fetch_account(&self) -> KahoResult<Account> {
        self.get(Endpoint::Account.path()).await
    }

    /// Calls the Stoat API or client internals to disable account for this resource.
    pub async fn disable_account(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountDisable.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to change password for this resource.
    pub async fn change_password(&self, payload: impl Into<AccountChangePassword>) -> KahoResult {
        self.patch_empty(Endpoint::AccountChangePassword.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to change email for this resource.
    pub async fn change_email(&self, payload: impl Into<AccountChangeEmail>) -> KahoResult {
        self.patch_empty(Endpoint::AccountChangeEmail.path(), payload.into())
            .await
    }

    /// Verify an account email with a verification code.
    pub async fn verify_email(&self, code: &str) -> KahoResult {
        self.post_empty(Endpoint::AccountVerify(code.to_owned()).path(), ())
            .await
    }

    /// Calls the Stoat API or client internals to send password reset for this resource.
    pub async fn send_password_reset(
        &self,
        payload: impl Into<AccountSendPasswordReset>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountResetPassword.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to reset password for this resource.
    pub async fn reset_password(&self, payload: impl Into<AccountPasswordReset>) -> KahoResult {
        self.patch_empty(Endpoint::AccountResetPassword.path(), payload.into())
            .await
    }

    // Bot-related methods
    /// Calls the Stoat API or client internals to fetch public bot for this resource.
    pub async fn fetch_public_bot(&self, bot_id: &str) -> KahoResult<PublicBot> {
        self.get(Endpoint::BotInvite(bot_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to create bot for this resource.
    pub async fn create_bot(&self, payload: impl Into<BotCreate>) -> KahoResult<BotCreateResponse> {
        self.post(Endpoint::BotCreate.path(), payload.into()).await
    }

    /// Calls the Stoat API or client internals to invite bot for this resource.
    pub async fn invite_bot(
        &self,
        bot_id: &str,
        payload: impl Into<BotInvite>,
    ) -> KahoResult<BotInviteResponse> {
        self.post(
            Endpoint::BotInvite(bot_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch bot for this resource.
    pub async fn fetch_bot(&self, bot_id: &str) -> KahoResult<Bot> {
        self.get(Endpoint::Bot(bot_id.to_owned()).path()).await
    }

    /// Calls the Stoat API or client internals to delete bot for this resource.
    pub async fn delete_bot(&self, bot_id: &str) -> KahoResult {
        self.delete(Endpoint::Bot(bot_id.to_owned()).path(), None::<()>)
            .await
    }

    /// Calls the Stoat API or client internals to edit bot for this resource.
    pub async fn edit_bot(&self, bot_id: &str, payload: impl Into<BotUpdate>) -> KahoResult<Bot> {
        self.patch(Endpoint::Bot(bot_id.to_owned()).path(), payload.into())
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

    /// Calls the Stoat API or client internals to edit user for this resource.
    pub async fn edit_user(
        &self,
        user_id: &str,
        payload: impl Into<UserUpdate>,
    ) -> KahoResult<User> {
        self.patch(Endpoint::User(user_id.to_owned()).path(), payload.into())
            .await
    }

    /// Get properties of the targeted user.
    pub async fn fetch_user(&self, user_id: &str) -> KahoResult<User> {
        self.get(Endpoint::User(user_id.to_owned()).path()).await
    }

    /// Get the flags of the targeted user.
    pub async fn fetch_user_flags(&self, user_id: &str) -> KahoResult<FlagResponse> {
        self.get(Endpoint::UserFlags(user_id.to_owned()).path())
            .await
    }

    /// Change the current user's username.
    pub async fn change_username(&self, payload: impl Into<ChangeUsername>) -> KahoResult<User> {
        self.patch(Endpoint::UserUsername.path(), payload.into())
            .await
    }

    /// Fetch the default avatar for a user.
    pub async fn fetch_default_avatar(&self, user_id: &str) -> KahoResult<DefaultAvatar> {
        let bytes = self
            .get_bytes(Endpoint::UserDefaultAvatar(user_id.to_owned()).path())
            .await?;

        Ok(DefaultAvatar { bytes })
    }

    /// Calls the Stoat API or client internals to fetch user profile for this resource.
    pub async fn fetch_user_profile(&self, user_id: &str) -> KahoResult<UserProfile> {
        self.get(Endpoint::UserProfile(user_id.to_owned()).path())
            .await
    }

    /// Fetch mutual friends, servers, groups, and DMs for a user.
    pub async fn fetch_mutual_relationships(&self, user_id: &str) -> KahoResult<MutualResponse> {
        self.get(Endpoint::RelationshipMutual(user_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to accept friend request for this resource.
    pub async fn accept_friend_request(&self, user_id: &str) -> KahoResult<User> {
        self.put_return(Endpoint::RelationshipFriend(user_id.to_owned()).path(), ())
            .await
    }

    /// Deny a friend request or remove a friend.
    pub async fn remove_friend(&self, user_id: &str) -> KahoResult<User> {
        self.delete_return(
            Endpoint::RelationshipFriend(user_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to block user for this resource.
    pub async fn block_user(&self, user_id: &str) -> KahoResult<User> {
        self.put_return(Endpoint::RelationshipBlock(user_id.to_owned()).path(), ())
            .await
    }

    /// Calls the Stoat API or client internals to unblock user for this resource.
    pub async fn unblock_user(&self, user_id: &str) -> KahoResult<User> {
        self.delete_return(
            Endpoint::RelationshipBlock(user_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to send friend request for this resource.
    pub async fn send_friend_request(
        &self,
        payload: impl Into<SendFriendRequest>,
    ) -> KahoResult<User> {
        self.post(Endpoint::RelationshipFriends.path(), payload.into())
            .await
    }

    // Custom emoji methods
    /// Calls the Stoat API or client internals to fetch emoji for this resource.
    pub async fn fetch_emoji(&self, emoji_id: &str) -> KahoResult<Emoji> {
        self.get(Endpoint::Emoji(emoji_id.to_owned()).path()).await
    }

    /// Calls the Stoat API or client internals to create emoji for this resource.
    pub async fn create_emoji(
        &self,
        emoji_id: &str,
        payload: impl Into<EmojiCreate>,
    ) -> KahoResult<Emoji> {
        self.put_return(Endpoint::Emoji(emoji_id.to_owned()).path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to delete emoji for this resource.
    pub async fn delete_emoji(&self, emoji_id: &str) -> KahoResult {
        self.delete(Endpoint::Emoji(emoji_id.to_owned()).path(), None::<()>)
            .await
    }

    // Invite methods
    /// Calls the Stoat API or client internals to fetch invite for this resource.
    pub async fn fetch_invite(&self, invite_id: &str) -> KahoResult<Invite> {
        self.get(Endpoint::Invite(invite_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to accept invite for this resource.
    pub async fn accept_invite(&self, invite_id: &str) -> KahoResult<InviteJoinResponse> {
        self.post(Endpoint::Invite(invite_id.to_owned()).path(), ())
            .await
    }

    /// Calls the Stoat API or client internals to delete invite for this resource.
    pub async fn delete_invite(&self, invite_id: &str) -> KahoResult {
        self.delete(Endpoint::Invite(invite_id.to_owned()).path(), None::<()>)
            .await
    }

    // Message-related methods
    /// Acknowledge a message in the specified channel.
    pub async fn acknowledge_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.put(
            Endpoint::ChannelMessageAck(channel_id.to_owned(), message_id.to_owned()).path(),
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
        let mut path = Endpoint::ChannelMessages(channel_id.to_owned()).path();

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
            Endpoint::ChannelMessages(channel_id.to_owned()).path(),
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
            Endpoint::ChannelMessageSearch(channel_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Pin a message in the specified channel.
    pub async fn pin_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.post(
            Endpoint::ChannelMessagePin(channel_id.to_owned(), message_id.to_owned()).path(),
            (),
        )
        .await
    }

    /// Unpin a message in the specified channel.
    pub async fn unpin_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ChannelMessagePin(channel_id.to_owned(), message_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Fetch a message in the specified channel.
    pub async fn fetch_message(&self, channel_id: &str, message_id: &str) -> KahoResult<Message> {
        self.get(Endpoint::ChannelMessage(channel_id.to_owned(), message_id.to_owned()).path())
            .await
    }

    /// Delete a message in the specified channel.
    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ChannelMessage(channel_id.to_owned(), message_id.to_owned()).path(),
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
            Endpoint::ChannelMessage(channel_id.to_owned(), message_id.to_owned()).path(),
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
            Endpoint::ChannelMessageBulk(channel_id.to_owned()).path(),
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
            id: message_id.to_owned().into(),
            mention,
            fail_if_not_exists: true,
        };

        let mut message_payload: MessageSend = payload.into();
        message_payload.replies.push(reply_intent);

        self.send_message(channel_id, message_payload).await
    }
}

// Additional typed Stoat API coverage.
impl HttpClient {
    /// Calls the Stoat API or client internals to fetch instance config for this resource.
    pub async fn fetch_instance_config(&self) -> KahoResult<InstanceConfig> {
        self.get(Endpoint::InstanceConfig.path()).await
    }

    /// Fetch direct message channels for the current user.
    pub async fn fetch_direct_message_channels(&self) -> KahoResult<Vec<Channel>> {
        self.get(Endpoint::UserDMs.path()).await
    }

    /// Open a direct message channel with a user.
    pub async fn open_direct_message(&self, user_id: &str) -> KahoResult<Channel> {
        self.get(Endpoint::UserDM(user_id.to_owned()).path()).await
    }

    /// Calls the Stoat API or client internals to report safety for this resource.
    pub async fn report_safety(&self, payload: impl Into<SafetyReportCreate>) -> KahoResult {
        self.post_empty(Endpoint::UserSafety.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to fetch channel for this resource.
    pub async fn fetch_channel(&self, channel_id: &str) -> KahoResult<Channel> {
        self.get(Endpoint::Channel(channel_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to close channel for this resource.
    pub async fn close_channel(
        &self,
        channel_id: &str,
        query: impl Into<Option<ChannelCloseQuery>>,
    ) -> KahoResult {
        let mut path = Endpoint::Channel(channel_id.to_owned()).path();
        if let Some(q) = query.into() {
            let encoded = serde_urlencoded::to_string(q).unwrap_or_default();
            if !encoded.is_empty() {
                path.push('?');
                path.push_str(&encoded);
            }
        }
        self.delete(path, None::<()>).await
    }

    /// Calls the Stoat API or client internals to edit channel for this resource.
    pub async fn edit_channel(
        &self,
        channel_id: &str,
        payload: impl Into<ChannelUpdate>,
    ) -> KahoResult<Channel> {
        self.patch(
            Endpoint::Channel(channel_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to create channel invite for this resource.
    pub async fn create_channel_invite(&self, channel_id: &str) -> KahoResult<Invite> {
        self.post(Endpoint::ChannelInvites(channel_id.to_owned()).path(), ())
            .await
    }

    /// Calls the Stoat API or client internals to set channel permissions for this resource.
    pub async fn set_channel_permissions(
        &self,
        channel_id: &str,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.put(
            Endpoint::ChannelPermission(channel_id.to_owned(), role_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Calls the Stoat API or client internals to set channel default permissions for this resource.
    pub async fn set_channel_default_permissions(
        &self,
        channel_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.put(
            Endpoint::ChannelPermissionDefault(channel_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Calls the Stoat API or client internals to add reaction for this resource.
    pub async fn add_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> KahoResult {
        self.put(
            Endpoint::ChannelMessageReaction(
                channel_id.to_owned(),
                message_id.to_owned(),
                emoji.to_owned(),
            )
            .path(),
            (),
        )
        .await
    }

    /// Calls the Stoat API or client internals to remove reaction for this resource.
    pub async fn remove_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> KahoResult {
        self.delete(
            Endpoint::ChannelMessageReaction(
                channel_id.to_owned(),
                message_id.to_owned(),
                emoji.to_owned(),
            )
            .path(),
            None::<()>,
        )
        .await
    }

    /// Remove all reactions from a message, or one emoji's reactions when supplied by the API payload.
    pub async fn clear_reactions(&self, channel_id: &str, message_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ChannelMessageReactions(channel_id.to_owned(), message_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch group members for this resource.
    pub async fn fetch_group_members(&self, channel_id: &str) -> KahoResult<Vec<User>> {
        self.get(Endpoint::ChannelMembers(channel_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to create group for this resource.
    pub async fn create_group(&self, payload: impl Into<GroupCreate>) -> KahoResult<Channel> {
        self.post(Endpoint::ChannelCreate.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to add group recipient for this resource.
    pub async fn add_group_recipient(&self, group_id: &str, member_id: &str) -> KahoResult {
        self.put(
            Endpoint::ChannelRecipient(group_id.to_owned(), member_id.to_owned()).path(),
            (),
        )
        .await
    }

    /// Calls the Stoat API or client internals to remove group recipient for this resource.
    pub async fn remove_group_recipient(&self, group_id: &str, member_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ChannelRecipient(group_id.to_owned(), member_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to join call for this resource.
    pub async fn join_call(&self, channel_id: &str) -> KahoResult<JoinCallResponse> {
        self.post(Endpoint::ChannelJoinCall(channel_id.to_owned()).path(), ())
            .await
    }

    /// Calls the Stoat API or client internals to end ring for this resource.
    pub async fn end_ring(&self, channel_id: &str, user_id: &str) -> KahoResult {
        self.put(
            Endpoint::ChannelEndRing(channel_id.to_owned(), user_id.to_owned()).path(),
            (),
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch channel webhooks for this resource.
    pub async fn fetch_channel_webhooks(&self, channel_id: &str) -> KahoResult<Vec<Webhook>> {
        self.get(Endpoint::ChannelWebhooks(channel_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to create webhook for this resource.
    pub async fn create_webhook(
        &self,
        channel_id: &str,
        payload: impl Into<WebhookCreate>,
    ) -> KahoResult<Webhook> {
        self.post(
            Endpoint::ChannelWebhooks(channel_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch webhook with token for this resource.
    pub async fn fetch_webhook_with_token(
        &self,
        webhook_id: &str,
        token: &str,
    ) -> KahoResult<Webhook> {
        self.get(Endpoint::WebhookWithToken(webhook_id.to_owned(), token.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to execute webhook for this resource.
    pub async fn execute_webhook(
        &self,
        webhook_id: &str,
        token: &str,
        payload: impl Into<WebhookExecute>,
    ) -> KahoResult {
        self.post_empty(
            Endpoint::WebhookWithToken(webhook_id.to_owned(), token.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to delete webhook with token for this resource.
    pub async fn delete_webhook_with_token(&self, webhook_id: &str, token: &str) -> KahoResult {
        self.delete(
            Endpoint::WebhookWithToken(webhook_id.to_owned(), token.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to edit webhook with token for this resource.
    pub async fn edit_webhook_with_token(
        &self,
        webhook_id: &str,
        token: &str,
        payload: impl Into<WebhookUpdate>,
    ) -> KahoResult<Webhook> {
        self.patch(
            Endpoint::WebhookWithToken(webhook_id.to_owned(), token.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch webhook for this resource.
    pub async fn fetch_webhook(&self, webhook_id: &str) -> KahoResult<Webhook> {
        self.get(Endpoint::Webhook(webhook_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to delete webhook for this resource.
    pub async fn delete_webhook(&self, webhook_id: &str) -> KahoResult {
        self.delete(Endpoint::Webhook(webhook_id.to_owned()).path(), None::<()>)
            .await
    }

    /// Calls the Stoat API or client internals to edit webhook for this resource.
    pub async fn edit_webhook(
        &self,
        webhook_id: &str,
        payload: impl Into<WebhookUpdate>,
    ) -> KahoResult<Webhook> {
        self.patch(
            Endpoint::Webhook(webhook_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to execute github webhook for this resource.
    pub async fn execute_github_webhook(
        &self,
        webhook_id: &str,
        token: &str,
        payload: serde_json::Value,
    ) -> KahoResult {
        self.post_empty(
            Endpoint::WebhookGithub(webhook_id.to_owned(), token.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Calls the Stoat API or client internals to create server for this resource.
    pub async fn create_server(&self, payload: impl Into<ServerCreate>) -> KahoResult<Server> {
        self.post(Endpoint::ServerCreate.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to fetch server for this resource.
    pub async fn fetch_server(&self, server_id: &str) -> KahoResult<Server> {
        self.get(Endpoint::Server(server_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to delete server for this resource.
    pub async fn delete_server(&self, server_id: &str) -> KahoResult {
        self.delete(Endpoint::Server(server_id.to_owned()).path(), None::<()>)
            .await
    }

    /// Calls the Stoat API or client internals to edit server for this resource.
    pub async fn edit_server(
        &self,
        server_id: &str,
        payload: impl Into<ServerEdit>,
    ) -> KahoResult<Server> {
        self.patch(
            Endpoint::Server(server_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to acknowledge server for this resource.
    pub async fn acknowledge_server(&self, server_id: &str) -> KahoResult {
        self.put(Endpoint::ServerAck(server_id.to_owned()).path(), ())
            .await
    }

    /// Calls the Stoat API or client internals to create server channel for this resource.
    pub async fn create_server_channel(
        &self,
        server_id: &str,
        payload: impl Into<ChannelCreate>,
    ) -> KahoResult<Channel> {
        self.post(
            Endpoint::ServerChannels(server_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch server members for this resource.
    pub async fn fetch_server_members(
        &self,
        server_id: &str,
        query: impl Into<Option<FetchMembersQuery>>,
    ) -> KahoResult<MemberList> {
        let mut path = Endpoint::ServerMembers(server_id.to_owned()).path();
        if let Some(q) = query.into() {
            let encoded = serde_urlencoded::to_string(q).unwrap_or_default();
            if !encoded.is_empty() {
                path.push('?');
                path.push_str(&encoded);
            }
        }
        self.get(path).await
    }

    /// Calls the Stoat API or client internals to fetch server member for this resource.
    pub async fn fetch_server_member(
        &self,
        server_id: &str,
        member_id: &str,
    ) -> KahoResult<Member> {
        self.get(Endpoint::ServerMember(server_id.to_owned(), member_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to kick server member for this resource.
    pub async fn kick_server_member(&self, server_id: &str, member_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ServerMember(server_id.to_owned(), member_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to edit server member for this resource.
    pub async fn edit_server_member(
        &self,
        server_id: &str,
        member_id: &str,
        payload: impl Into<MemberUpdate>,
    ) -> KahoResult<Member> {
        self.patch(
            Endpoint::ServerMember(server_id.to_owned(), member_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Query server members using the experimental endpoint.
    pub async fn query_server_members(
        &self,
        server_id: &str,
        payload: impl Into<MembersExperimentalQuery>,
    ) -> KahoResult<MemberList> {
        self.get(format!(
            "{}?{}",
            Endpoint::ServerMemberExperimentalQuery(server_id.to_owned()).path(),
            serde_urlencoded::to_string(payload.into()).unwrap_or_default()
        ))
        .await
    }

    /// Calls the Stoat API or client internals to ban user for this resource.
    pub async fn ban_user(
        &self,
        server_id: &str,
        user_id: &str,
        payload: impl Into<BanCreate>,
    ) -> KahoResult {
        self.put(
            Endpoint::ServerBan(server_id.to_owned(), user_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to unban user for this resource.
    pub async fn unban_user(&self, server_id: &str, user_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ServerBan(server_id.to_owned(), user_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch server bans for this resource.
    pub async fn fetch_server_bans(&self, server_id: &str) -> KahoResult<ServerBans> {
        self.get(Endpoint::ServerBans(server_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to fetch server invites for this resource.
    pub async fn fetch_server_invites(&self, server_id: &str) -> KahoResult<Vec<Invite>> {
        self.get(Endpoint::ServerInvites(server_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to create server role for this resource.
    pub async fn create_server_role(
        &self,
        server_id: &str,
        payload: impl Into<RoleCreate>,
    ) -> KahoResult<RoleCreateResponse> {
        self.post(
            Endpoint::ServerRoles(server_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to fetch server role for this resource.
    pub async fn fetch_server_role(&self, server_id: &str, role_id: &str) -> KahoResult<Role> {
        self.get(Endpoint::ServerRole(server_id.to_owned(), role_id.to_owned()).path())
            .await
    }

    /// Calls the Stoat API or client internals to delete server role for this resource.
    pub async fn delete_server_role(&self, server_id: &str, role_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ServerRole(server_id.to_owned(), role_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Calls the Stoat API or client internals to edit server role for this resource.
    pub async fn edit_server_role(
        &self,
        server_id: &str,
        role_id: &str,
        payload: impl Into<RoleUpdate>,
    ) -> KahoResult<Role> {
        self.patch(
            Endpoint::ServerRole(server_id.to_owned(), role_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to set server permissions for this resource.
    pub async fn set_server_permissions(
        &self,
        server_id: &str,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.put(
            Endpoint::ServerPermission(server_id.to_owned(), role_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Calls the Stoat API or client internals to set server default permissions for this resource.
    pub async fn set_server_default_permissions(
        &self,
        server_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.put(
            Endpoint::ServerPermissionDefault(server_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Calls the Stoat API or client internals to set server role ranks for this resource.
    pub async fn set_server_role_ranks(
        &self,
        server_id: &str,
        payload: impl Into<RoleRanksUpdate>,
    ) -> KahoResult {
        self.patch_empty(
            Endpoint::ServerRoleRanks(server_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Calls the Stoat API or client internals to onboarding hello for this resource.
    pub async fn onboarding_hello(&self) -> KahoResult<OnboardingHello> {
        self.get(Endpoint::OnboardingHello.path()).await
    }

    /// Calls the Stoat API or client internals to complete onboarding for this resource.
    pub async fn complete_onboarding(
        &self,
        payload: impl Into<OnboardingComplete>,
    ) -> KahoResult<User> {
        self.post(Endpoint::OnboardingComplete.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to validate MFA ticket for this resource.
    pub async fn validate_mfa_ticket(
        &self,
        payload: impl Into<MfaTicketPayload>,
    ) -> KahoResult<MfaResponse> {
        self.put_return(Endpoint::MfaTicket.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to fetch MFA for this resource.
    pub async fn fetch_mfa(&self) -> KahoResult<MfaStatus> {
        self.get(Endpoint::Mfa.path()).await
    }

    /// Calls the Stoat API or client internals to create MFA recovery for this resource.
    pub async fn create_mfa_recovery(
        &self,
        payload: impl Into<MfaRecoveryPayload>,
    ) -> KahoResult<MfaResponse> {
        self.post(Endpoint::MfaRecovery.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to regenerate MFA recovery for this resource.
    pub async fn regenerate_mfa_recovery(
        &self,
        payload: impl Into<MfaRecoveryPayload>,
    ) -> KahoResult<MfaResponse> {
        self.patch(Endpoint::MfaRecovery.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to fetch MFA methods for this resource.
    pub async fn fetch_mfa_methods(&self) -> KahoResult<MfaMethods> {
        self.get(Endpoint::MfaMethods.path()).await
    }

    /// Calls the Stoat API or client internals to enable MFA TOTP for this resource.
    pub async fn enable_mfa_totp(
        &self,
        payload: impl Into<MfaTotpPayload>,
    ) -> KahoResult<MfaResponse> {
        self.put_return(Endpoint::MfaTotp.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to verify MFA TOTP for this resource.
    pub async fn verify_mfa_totp(
        &self,
        payload: impl Into<MfaTotpPayload>,
    ) -> KahoResult<MfaResponse> {
        self.post(Endpoint::MfaTotp.path(), payload.into()).await
    }

    /// Calls the Stoat API or client internals to disable MFA TOTP for this resource.
    pub async fn disable_mfa_totp(&self, payload: impl Into<MfaTotpPayload>) -> KahoResult {
        self.delete(Endpoint::MfaTotp.path(), Some(payload.into()))
            .await
    }

    /// Calls the Stoat API or client internals to fetch sync settings for this resource.
    pub async fn fetch_sync_settings(
        &self,
        payload: impl Into<SyncSettingsFetch>,
    ) -> KahoResult<SyncSettings> {
        self.post(Endpoint::SyncSettingsFetch.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to set sync setting for this resource.
    pub async fn set_sync_setting(&self, payload: impl Into<SyncSettingsSet>) -> KahoResult {
        self.post_empty(Endpoint::SyncSettingsSet.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to fetch unreads for this resource.
    pub async fn fetch_unreads(&self) -> KahoResult<Unreads> {
        self.get(Endpoint::SyncUnreads.path()).await
    }

    /// Calls the Stoat API or client internals to subscribe push for this resource.
    pub async fn subscribe_push(&self, payload: impl Into<PushSubscription>) -> KahoResult {
        self.post_empty(Endpoint::PushSubscribe.path(), payload.into())
            .await
    }

    /// Calls the Stoat API or client internals to unsubscribe push for this resource.
    pub async fn unsubscribe_push(&self, payload: impl Into<PushSubscription>) -> KahoResult {
        self.post_empty(Endpoint::PushUnsubscribe.path(), payload.into())
            .await
    }
}
