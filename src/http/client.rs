use {
    reqwest::{
        header::{HeaderMap, HeaderValue},
        multipart::{Form, Part},
        Client, Method, RequestBuilder, Response, StatusCode,
    },
    serde::{de::DeserializeOwned, ser::Serialize},
    serde_json::{json, Value},
    std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    },
    tracing::{debug, warn},
};

use crate::{
    error::{KahoError, KahoResult},
    http::{endpoint::Endpoint, HttpConfig, RateLimitedResponse, RateLimiter},
    models::*,
};

/// Point-in-time HTTP diagnostics useful for distinguishing Kaho-side waiting from Stoat/network latency.
#[derive(Clone, Debug, Default)]
pub struct HttpMetrics {
    /// Number of HTTP attempts started, including retries after a 429 response.
    pub requests_started: u64,
    /// Number of HTTP responses received.
    pub responses_received: u64,
    /// Number of successful non-429 responses received.
    pub successful_responses: u64,
    /// Number of non-success, non-429 responses received.
    pub failed_responses: u64,
    /// Number of request attempts that failed before receiving a response.
    pub transport_errors: u64,
    /// Number of 429 responses received from Stoat.
    pub rate_limit_hits: u64,
    /// Total time requests have spent waiting for locally tracked rate-limit capacity.
    pub rate_limit_wait: Duration,
    /// Time from starting the latest network request until response headers arrived.
    pub last_response_headers_latency: Duration,
    /// Total time spent decoding response bodies through Kaho helpers.
    pub body_decode_time: Duration,
    /// Number of response body decode/read errors.
    pub body_decode_errors: u64,
}

#[derive(Debug, Default)]
struct HttpMetricsInner {
    requests_started: AtomicU64,
    responses_received: AtomicU64,
    successful_responses: AtomicU64,
    failed_responses: AtomicU64,
    transport_errors: AtomicU64,
    rate_limit_hits: AtomicU64,
    rate_limit_wait_micros: AtomicU64,
    last_response_headers_micros: AtomicU64,
    body_decode_micros: AtomicU64,
    body_decode_errors: AtomicU64,
}

#[derive(Debug)]
struct HttpShared {
    client: Client,
    config: HttpConfig,
    rate_limiter: RateLimiter,
    metrics: HttpMetricsInner,
}

/// HTTP client for calling the Stoat REST API.
#[derive(Debug, Clone)]
pub struct HttpClient {
    shared: Arc<HttpShared>,
}

impl HttpClient {
    /// Create a new instance.
    pub fn new(config: HttpConfig) -> KahoResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("X-Bot-Token", HeaderValue::from_str(&config.token)?);

        let client = Client::builder()
            .default_headers(headers)
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .pool_idle_timeout(Some(config.pool_idle_timeout))
            .tcp_keepalive(Some(config.tcp_keepalive))
            .build()?;

        Ok(Self {
            shared: Arc::new(HttpShared {
                client,
                config,
                rate_limiter: RateLimiter::default(),
                metrics: HttpMetricsInner::default(),
            }),
        })
    }

    fn make_url(&self, path: impl AsRef<str>) -> String {
        format!(
            "{}/{}",
            self.shared.config.api_url.trim_end_matches('/'),
            path.as_ref().trim_start_matches('/')
        )
    }

    fn make_cdn_url(&self, tag: AttachmentTag) -> String {
        format!(
            "{}/{}",
            self.shared.config.cdn_url.trim_end_matches('/'),
            tag.as_str()
        )
    }

    async fn send_rate_limited(
        &self,
        method: Method,
        path: &str,
        build_request: impl Fn() -> RequestBuilder,
    ) -> KahoResult<Response> {
        loop {
            let waited = self.shared.rate_limiter.acquire(&method, path).await;
            self.shared
                .metrics
                .rate_limit_wait_micros
                .fetch_add(duration_to_micros(waited), Ordering::Relaxed);

            self.shared
                .metrics
                .requests_started
                .fetch_add(1, Ordering::Relaxed);
            let request_started = Instant::now();
            let response = match build_request().send().await {
                Ok(response) => response,
                Err(error) => {
                    self.shared
                        .metrics
                        .transport_errors
                        .fetch_add(1, Ordering::Relaxed);
                    self.shared.metrics.last_response_headers_micros.store(
                        duration_to_micros(request_started.elapsed()),
                        Ordering::Relaxed,
                    );
                    return Err(error.into());
                }
            };

            let response_latency = request_started.elapsed();
            self.shared
                .metrics
                .responses_received
                .fetch_add(1, Ordering::Relaxed);
            self.shared
                .metrics
                .last_response_headers_micros
                .store(duration_to_micros(response_latency), Ordering::Relaxed);
            self.shared
                .rate_limiter
                .update_from_headers(&method, path, response.headers())
                .await;

            let status = response.status();
            debug!(
                method = %method,
                path,
                status = status.as_u16(),
                response_headers_ms = response_latency.as_millis(),
                local_rate_limit_wait_ms = waited.as_millis(),
                "Stoat HTTP request completed"
            );

            if status == StatusCode::TOO_MANY_REQUESTS {
                self.shared
                    .metrics
                    .rate_limit_hits
                    .fetch_add(1, Ordering::Relaxed);
                let retry_after = self
                    .decode_json::<RateLimitedResponse>(response)
                    .await
                    .map(|payload| payload.retry_after)
                    .unwrap_or(10_000);

                warn!(
                    method = %method,
                    path,
                    retry_after_ms = retry_after,
                    "Stoat rate limit reached"
                );
                self.shared
                    .rate_limiter
                    .update_retry_after(&method, path, retry_after)
                    .await;
                continue;
            }

            if !status.is_success() {
                self.shared
                    .metrics
                    .failed_responses
                    .fetch_add(1, Ordering::Relaxed);
                return Err(KahoError::FailedRequest(response));
            }

            self.shared
                .metrics
                .successful_responses
                .fetch_add(1, Ordering::Relaxed);
            return Ok(response);
        }
    }

    fn record_body_decode<T>(
        &self,
        started: Instant,
        result: Result<T, reqwest::Error>,
    ) -> KahoResult<T> {
        self.shared
            .metrics
            .body_decode_micros
            .fetch_add(duration_to_micros(started.elapsed()), Ordering::Relaxed);

        result.map_err(|error| {
            self.shared
                .metrics
                .body_decode_errors
                .fetch_add(1, Ordering::Relaxed);
            error.into()
        })
    }

    async fn decode_json<T: DeserializeOwned>(&self, response: Response) -> KahoResult<T> {
        let started = Instant::now();
        let result = response.json().await;
        self.record_body_decode(started, result)
    }

    async fn decode_bytes(&self, response: Response) -> KahoResult<Vec<u8>> {
        let started = Instant::now();
        let result = response.bytes().await;
        self.record_body_decode(started, result)
            .map(|bytes| bytes.to_vec())
    }

    /// Return a point-in-time HTTP diagnostics snapshot.
    pub fn metrics(&self) -> HttpMetrics {
        HttpMetrics {
            requests_started: self.shared.metrics.requests_started.load(Ordering::Relaxed),
            responses_received: self
                .shared
                .metrics
                .responses_received
                .load(Ordering::Relaxed),
            successful_responses: self
                .shared
                .metrics
                .successful_responses
                .load(Ordering::Relaxed),
            failed_responses: self.shared.metrics.failed_responses.load(Ordering::Relaxed),
            transport_errors: self.shared.metrics.transport_errors.load(Ordering::Relaxed),
            rate_limit_hits: self.shared.metrics.rate_limit_hits.load(Ordering::Relaxed),
            rate_limit_wait: Duration::from_micros(
                self.shared
                    .metrics
                    .rate_limit_wait_micros
                    .load(Ordering::Relaxed),
            ),
            last_response_headers_latency: Duration::from_micros(
                self.shared
                    .metrics
                    .last_response_headers_micros
                    .load(Ordering::Relaxed),
            ),
            body_decode_time: Duration::from_micros(
                self.shared
                    .metrics
                    .body_decode_micros
                    .load(Ordering::Relaxed),
            ),
            body_decode_errors: self
                .shared
                .metrics
                .body_decode_errors
                .load(Ordering::Relaxed),
        }
    }

    /// Send a GET request and deserialize the JSON response.
    pub async fn get<T: DeserializeOwned>(&self, path: impl AsRef<str>) -> KahoResult<T> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::GET, path, || {
                self.shared.client.get(self.make_url(path))
            })
            .await?;

        self.decode_json(response).await
    }

    async fn get_query<T: DeserializeOwned, Q: Serialize + ?Sized>(
        &self,
        path: impl AsRef<str>,
        query: &Q,
    ) -> KahoResult<T> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::GET, path, || {
                self.shared.client.get(self.make_url(path)).query(query)
            })
            .await?;

        self.decode_json(response).await
    }

    async fn delete_query<Q: Serialize + ?Sized>(
        &self,
        path: impl AsRef<str>,
        query: &Q,
    ) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::DELETE, path, || {
            self.shared.client.delete(self.make_url(path)).query(query)
        })
        .await?;
        Ok(())
    }

    /// Send a GET request and return the raw response bytes.
    pub async fn get_bytes(&self, path: impl AsRef<str>) -> KahoResult<Vec<u8>> {
        let path = path.as_ref();
        let response = self
            .send_rate_limited(Method::GET, path, || {
                self.shared.client.get(self.make_url(path))
            })
            .await?;

        self.decode_bytes(response).await
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
                self.shared.client.post(self.make_url(path)).json(&payload)
            })
            .await?;

        self.decode_json(response).await
    }

    /// Send a POST request with a JSON payload and ignore the response body.
    pub async fn post_empty<U: Serialize>(&self, path: impl AsRef<str>, payload: U) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::POST, path, || {
            self.shared.client.post(self.make_url(path)).json(&payload)
        })
        .await?;

        Ok(())
    }

    /// Send a PATCH request with a JSON payload and ignore the response body.
    pub async fn patch_empty<U: Serialize>(&self, path: impl AsRef<str>, payload: U) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::PATCH, path, || {
            self.shared.client.patch(self.make_url(path)).json(&payload)
        })
        .await?;

        Ok(())
    }

    /// Send a PUT request with a JSON payload.
    pub async fn put<T: Serialize>(&self, path: impl AsRef<str>, payload: T) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::PUT, path, || {
            self.shared.client.put(self.make_url(path)).json(&payload)
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
                self.shared.client.put(self.make_url(path)).json(&payload)
            })
            .await?;

        self.decode_json(response).await
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
                self.shared.client.patch(self.make_url(path)).json(&payload)
            })
            .await?;

        self.decode_json(response).await
    }

    /// Upload raw file bytes to the Stoat CDN and return the generated file ID.
    ///
    /// The returned ID can be used in message attachments, embed media, avatars,
    /// icons, banners, and other API fields that accept uploaded file IDs.
    pub async fn upload_file(
        &self,
        tag: AttachmentTag,
        filename: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> KahoResult<Id> {
        let filename = filename.into();
        let bytes = bytes.into();
        let path = format!("/{}", tag.as_str());
        let url = self.make_cdn_url(tag);

        let response = self
            .send_rate_limited(Method::POST, &path, || {
                let part = Part::bytes(bytes.clone()).file_name(filename.clone());
                let form = Form::new().part("file", part);
                self.shared.client.post(url.clone()).multipart(form)
            })
            .await?;

        Ok(self.decode_json::<FileUploadResponse>(response).await?.id)
    }

    /// Send a DELETE request.
    pub async fn delete<T: Serialize>(
        &self,
        path: impl AsRef<str>,
        payload: Option<T>,
    ) -> KahoResult {
        let path = path.as_ref();
        self.send_rate_limited(Method::DELETE, path, || {
            let mut request = self.shared.client.delete(self.make_url(path));

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
                let mut request = self.shared.client.delete(self.make_url(path));

                if let Some(payload) = payload.as_ref() {
                    request = request.json(payload);
                }

                request
            })
            .await?;

        self.decode_json(response).await
    }

    // Account-related methods
    /// Create an account.
    pub async fn create_account(&self, payload: impl Into<AccountCreate>) -> KahoResult {
        self.post_empty(Endpoint::AccountCreate.path(), payload.into())
            .await
    }

    /// Resend the account verification email.
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

    /// Delete the account.
    pub async fn delete_account(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountDelete.path(), payload.into())
            .await
    }

    /// Fetch the account.
    pub async fn fetch_account(&self) -> KahoResult<Account> {
        self.get(Endpoint::Account.path()).await
    }

    /// Disable the account.
    pub async fn disable_account(
        &self,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        self.post_empty(Endpoint::AccountDisable.path(), payload.into())
            .await
    }

    /// Change the account password.
    pub async fn change_password(&self, payload: impl Into<AccountChangePassword>) -> KahoResult {
        self.patch_empty(Endpoint::AccountChangePassword.path(), payload.into())
            .await
    }

    /// Change the account email address.
    pub async fn change_email(&self, payload: impl Into<AccountChangeEmail>) -> KahoResult {
        self.patch_empty(Endpoint::AccountChangeEmail.path(), payload.into())
            .await
    }

    /// Verify an account email with a verification code.
    pub async fn verify_email(&self, code: &str) -> KahoResult {
        self.post_empty(Endpoint::AccountVerify(code.to_owned()).path(), ())
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

    /// Reset the account password.
    pub async fn reset_password(&self, payload: impl Into<AccountPasswordReset>) -> KahoResult {
        self.patch_empty(Endpoint::AccountResetPassword.path(), payload.into())
            .await
    }

    // Bot-related methods
    /// Fetch a public bot.
    pub async fn fetch_public_bot(&self, bot_id: &str) -> KahoResult<PublicBot> {
        self.get(Endpoint::BotInvite(bot_id.to_owned()).path())
            .await
    }

    /// Create a bot.
    pub async fn create_bot(&self, payload: impl Into<BotCreate>) -> KahoResult<BotCreateResponse> {
        self.post(Endpoint::BotCreate.path(), payload.into()).await
    }

    /// Invite a bot.
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

    /// Fetch the bot.
    pub async fn fetch_bot(&self, bot_id: &str) -> KahoResult<Bot> {
        self.get(Endpoint::Bot(bot_id.to_owned()).path()).await
    }

    /// Delete the bot.
    pub async fn delete_bot(&self, bot_id: &str) -> KahoResult {
        self.delete(Endpoint::Bot(bot_id.to_owned()).path(), None::<()>)
            .await
    }

    /// Edit the bot.
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

    /// Edit the user.
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

    /// Fetch the user profile.
    pub async fn fetch_user_profile(&self, user_id: &str) -> KahoResult<UserProfile> {
        self.get(Endpoint::UserProfile(user_id.to_owned()).path())
            .await
    }

    /// Fetch mutual friends, servers, groups, and DMs for a user.
    pub async fn fetch_mutual_relationships(&self, user_id: &str) -> KahoResult<MutualResponse> {
        self.get(Endpoint::RelationshipMutual(user_id.to_owned()).path())
            .await
    }

    /// Accept a friend request.
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

    /// Block a user.
    pub async fn block_user(&self, user_id: &str) -> KahoResult<User> {
        self.put_return(Endpoint::RelationshipBlock(user_id.to_owned()).path(), ())
            .await
    }

    /// Unblock a user.
    pub async fn unblock_user(&self, user_id: &str) -> KahoResult<User> {
        self.delete_return(
            Endpoint::RelationshipBlock(user_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Send a friend request.
    pub async fn send_friend_request(
        &self,
        payload: impl Into<SendFriendRequest>,
    ) -> KahoResult<User> {
        self.post(Endpoint::RelationshipFriends.path(), payload.into())
            .await
    }

    // Custom emoji methods
    /// Fetch the emoji.
    pub async fn fetch_emoji(&self, emoji_id: &str) -> KahoResult<Emoji> {
        self.get(Endpoint::Emoji(emoji_id.to_owned()).path()).await
    }

    /// Create an emoji.
    pub async fn create_emoji(
        &self,
        emoji_id: &str,
        payload: impl Into<EmojiCreate>,
    ) -> KahoResult<Emoji> {
        self.put_return(Endpoint::Emoji(emoji_id.to_owned()).path(), payload.into())
            .await
    }

    /// Delete the emoji.
    pub async fn delete_emoji(&self, emoji_id: &str) -> KahoResult {
        self.delete(Endpoint::Emoji(emoji_id.to_owned()).path(), None::<()>)
            .await
    }

    // Invite methods
    /// Fetch the invite.
    pub async fn fetch_invite(&self, invite_id: &str) -> KahoResult<Invite> {
        self.get(Endpoint::Invite(invite_id.to_owned()).path())
            .await
    }

    /// Accept the invite.
    pub async fn accept_invite(&self, invite_id: &str) -> KahoResult<InviteJoinResponse> {
        self.post(Endpoint::Invite(invite_id.to_owned()).path(), ())
            .await
    }

    /// Delete the invite.
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
        let path = Endpoint::ChannelMessages(channel_id.to_owned()).path();
        match query.into() {
            Some(query) => self.get_query(path, &query).await,
            None => self.get(path).await,
        }
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
        let payload = json!({ "ids": message_ids });

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
    /// Fetch the instance configuration.
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

    /// Submit a safety report.
    pub async fn report_safety(&self, payload: impl Into<SafetyReportCreate>) -> KahoResult {
        self.post_empty(Endpoint::UserSafety.path(), payload.into())
            .await
    }

    /// Fetch the channel.
    pub async fn fetch_channel(&self, channel_id: &str) -> KahoResult<Channel> {
        self.get(Endpoint::Channel(channel_id.to_owned()).path())
            .await
    }

    /// Close the channel.
    pub async fn close_channel(
        &self,
        channel_id: &str,
        query: impl Into<Option<ChannelCloseQuery>>,
    ) -> KahoResult {
        let path = Endpoint::Channel(channel_id.to_owned()).path();
        match query.into() {
            Some(query) => self.delete_query(path, &query).await,
            None => self.delete(path, None::<()>).await,
        }
    }

    /// Edit the channel.
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

    /// Create a channel invite.
    pub async fn create_channel_invite(&self, channel_id: &str) -> KahoResult<Invite> {
        self.post(Endpoint::ChannelInvites(channel_id.to_owned()).path(), ())
            .await
    }

    /// Set channel permissions.
    pub async fn set_channel_permissions(
        &self,
        channel_id: &str,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.set_channel_permissions_returning(channel_id, role_id, payload)
            .await
            .map(|_| ())
    }

    pub(crate) async fn set_channel_permissions_returning(
        &self,
        channel_id: &str,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult<Channel> {
        self.put_return(
            Endpoint::ChannelPermission(channel_id.to_owned(), role_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Set default channel permissions.
    pub async fn set_channel_default_permissions(
        &self,
        channel_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.set_channel_default_permissions_returning(channel_id, payload)
            .await
            .map(|_| ())
    }

    pub(crate) async fn set_channel_default_permissions_returning(
        &self,
        channel_id: &str,
        payload: OverrideField,
    ) -> KahoResult<Channel> {
        self.put_return(
            Endpoint::ChannelPermissionDefault(channel_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Add a reaction.
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

    /// Remove a reaction.
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

    /// Fetch group members.
    pub async fn fetch_group_members(&self, channel_id: &str) -> KahoResult<Vec<User>> {
        self.get(Endpoint::ChannelMembers(channel_id.to_owned()).path())
            .await
    }

    /// Create a group.
    pub async fn create_group(&self, payload: impl Into<GroupCreate>) -> KahoResult<Channel> {
        self.post(Endpoint::ChannelCreate.path(), payload.into())
            .await
    }

    /// Add a recipient to the group.
    pub async fn add_group_recipient(&self, group_id: &str, member_id: &str) -> KahoResult {
        self.put(
            Endpoint::ChannelRecipient(group_id.to_owned(), member_id.to_owned()).path(),
            (),
        )
        .await
    }

    /// Remove a recipient from the group.
    pub async fn remove_group_recipient(&self, group_id: &str, member_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ChannelRecipient(group_id.to_owned(), member_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Join a call.
    pub async fn join_call(&self, channel_id: &str) -> KahoResult<JoinCallResponse> {
        self.post(Endpoint::ChannelJoinCall(channel_id.to_owned()).path(), ())
            .await
    }

    /// End an outgoing ring.
    pub async fn end_ring(&self, channel_id: &str, user_id: &str) -> KahoResult {
        self.put(
            Endpoint::ChannelEndRing(channel_id.to_owned(), user_id.to_owned()).path(),
            (),
        )
        .await
    }

    /// Fetch channel webhooks.
    pub async fn fetch_channel_webhooks(&self, channel_id: &str) -> KahoResult<Vec<Webhook>> {
        self.get(Endpoint::ChannelWebhooks(channel_id.to_owned()).path())
            .await
    }

    /// Create a webhook.
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

    /// Fetch a webhook using its token.
    pub async fn fetch_webhook_with_token(
        &self,
        webhook_id: &str,
        token: &str,
    ) -> KahoResult<Webhook> {
        self.get(Endpoint::WebhookWithToken(webhook_id.to_owned(), token.to_owned()).path())
            .await
    }

    /// Execute a webhook.
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

    /// Delete a webhook using its token.
    pub async fn delete_webhook_with_token(&self, webhook_id: &str, token: &str) -> KahoResult {
        self.delete(
            Endpoint::WebhookWithToken(webhook_id.to_owned(), token.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Edit a webhook using its token.
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

    /// Fetch the webhook.
    pub async fn fetch_webhook(&self, webhook_id: &str) -> KahoResult<Webhook> {
        self.get(Endpoint::Webhook(webhook_id.to_owned()).path())
            .await
    }

    /// Delete the webhook.
    pub async fn delete_webhook(&self, webhook_id: &str) -> KahoResult {
        self.delete(Endpoint::Webhook(webhook_id.to_owned()).path(), None::<()>)
            .await
    }

    /// Edit the webhook.
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

    /// Execute a GitHub webhook.
    pub async fn execute_github_webhook(
        &self,
        webhook_id: &str,
        token: &str,
        payload: Value,
    ) -> KahoResult {
        self.post_empty(
            Endpoint::WebhookGithub(webhook_id.to_owned(), token.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Create a server.
    pub async fn create_server(&self, payload: impl Into<ServerCreate>) -> KahoResult<Server> {
        self.post(Endpoint::ServerCreate.path(), payload.into())
            .await
    }

    /// Fetch the server.
    pub async fn fetch_server(&self, server_id: &str) -> KahoResult<Server> {
        self.get(Endpoint::Server(server_id.to_owned()).path())
            .await
    }

    /// Delete the server.
    pub async fn delete_server(&self, server_id: &str) -> KahoResult {
        self.delete(Endpoint::Server(server_id.to_owned()).path(), None::<()>)
            .await
    }

    /// Edit the server.
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

    /// Acknowledge the server.
    pub async fn acknowledge_server(&self, server_id: &str) -> KahoResult {
        self.put(Endpoint::ServerAck(server_id.to_owned()).path(), ())
            .await
    }

    /// Create a server channel.
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

    /// Fetch server members.
    pub async fn fetch_server_members(
        &self,
        server_id: &str,
        query: impl Into<Option<FetchMembersQuery>>,
    ) -> KahoResult<MemberList> {
        let path = Endpoint::ServerMembers(server_id.to_owned()).path();
        match query.into() {
            Some(query) => self.get_query(path, &query).await,
            None => self.get(path).await,
        }
    }

    /// Fetch a server member.
    ///
    /// Stoat models this endpoint as a union: a plain `Member`, or a wrapper
    /// containing the member plus expanded roles when `roles=true` is used.
    /// This convenience method always returns the member object.
    pub async fn fetch_server_member(
        &self,
        server_id: &str,
        member_id: &str,
    ) -> KahoResult<Member> {
        Ok(self
            .fetch_server_member_response(server_id, member_id, false)
            .await?
            .into_member())
    }

    /// Fetch a server member, optionally expanding the roles referenced by it.
    pub async fn fetch_server_member_response(
        &self,
        server_id: &str,
        member_id: &str,
        include_roles: bool,
    ) -> KahoResult<MemberResponse> {
        let path = Endpoint::ServerMember(server_id.to_owned(), member_id.to_owned()).path();
        if include_roles {
            let query = [("roles", true)];
            self.get_query(path, &query).await
        } else {
            self.get(path).await
        }
    }

    /// Kick a server member.
    pub async fn kick_server_member(&self, server_id: &str, member_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ServerMember(server_id.to_owned(), member_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Edit a server member.
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
        let path = Endpoint::ServerMemberExperimentalQuery(server_id.to_owned()).path();
        let query = payload.into();
        self.get_query(path, &query).await
    }

    /// Ban a user.
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

    /// Unban a user.
    pub async fn unban_user(&self, server_id: &str, user_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ServerBan(server_id.to_owned(), user_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Fetch server bans.
    pub async fn fetch_server_bans(&self, server_id: &str) -> KahoResult<ServerBans> {
        self.get(Endpoint::ServerBans(server_id.to_owned()).path())
            .await
    }

    /// Fetch server invites.
    pub async fn fetch_server_invites(&self, server_id: &str) -> KahoResult<Vec<Invite>> {
        self.get(Endpoint::ServerInvites(server_id.to_owned()).path())
            .await
    }

    /// Create a server role.
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

    /// Fetch a server role.
    pub async fn fetch_server_role(&self, server_id: &str, role_id: &str) -> KahoResult<Role> {
        self.get(Endpoint::ServerRole(server_id.to_owned(), role_id.to_owned()).path())
            .await
    }

    /// Delete a server role.
    pub async fn delete_server_role(&self, server_id: &str, role_id: &str) -> KahoResult {
        self.delete(
            Endpoint::ServerRole(server_id.to_owned(), role_id.to_owned()).path(),
            None::<()>,
        )
        .await
    }

    /// Edit a server role.
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

    /// Set server permissions.
    pub async fn set_server_permissions(
        &self,
        server_id: &str,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.set_server_permissions_returning(server_id, role_id, payload)
            .await
            .map(|_| ())
    }

    pub(crate) async fn set_server_permissions_returning(
        &self,
        server_id: &str,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult<Server> {
        self.put_return(
            Endpoint::ServerPermission(server_id.to_owned(), role_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Set default server permissions.
    pub async fn set_server_default_permissions(
        &self,
        server_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        self.set_server_default_permissions_returning(server_id, payload)
            .await
            .map(|_| ())
    }

    pub(crate) async fn set_server_default_permissions_returning(
        &self,
        server_id: &str,
        payload: OverrideField,
    ) -> KahoResult<Server> {
        self.put_return(
            Endpoint::ServerPermissionDefault(server_id.to_owned()).path(),
            payload,
        )
        .await
    }

    /// Set server role ranks.
    pub async fn set_server_role_ranks(
        &self,
        server_id: &str,
        payload: impl Into<RoleRanksUpdate>,
    ) -> KahoResult {
        self.set_server_role_ranks_returning(server_id, payload)
            .await
            .map(|_| ())
    }

    pub(crate) async fn set_server_role_ranks_returning(
        &self,
        server_id: &str,
        payload: impl Into<RoleRanksUpdate>,
    ) -> KahoResult<Server> {
        self.patch(
            Endpoint::ServerRoleRanks(server_id.to_owned()).path(),
            payload.into(),
        )
        .await
    }

    /// Fetch onboarding information.
    pub async fn onboarding_hello(&self) -> KahoResult<OnboardingHello> {
        self.get(Endpoint::OnboardingHello.path()).await
    }

    /// Complete onboarding.
    pub async fn complete_onboarding(
        &self,
        payload: impl Into<OnboardingComplete>,
    ) -> KahoResult<User> {
        self.post(Endpoint::OnboardingComplete.path(), payload.into())
            .await
    }

    /// Validate an MFA ticket.
    pub async fn validate_mfa_ticket(
        &self,
        payload: impl Into<MfaTicketPayload>,
    ) -> KahoResult<MfaResponse> {
        self.put_return(Endpoint::MfaTicket.path(), payload.into())
            .await
    }

    /// Fetch MFA configuration.
    pub async fn fetch_mfa(&self) -> KahoResult<MfaStatus> {
        self.get(Endpoint::Mfa.path()).await
    }

    /// Create MFA recovery codes.
    pub async fn create_mfa_recovery(
        &self,
        payload: impl Into<MfaRecoveryPayload>,
    ) -> KahoResult<MfaResponse> {
        self.post(Endpoint::MfaRecovery.path(), payload.into())
            .await
    }

    /// Regenerate MFA recovery codes.
    pub async fn regenerate_mfa_recovery(
        &self,
        payload: impl Into<MfaRecoveryPayload>,
    ) -> KahoResult<MfaResponse> {
        self.patch(Endpoint::MfaRecovery.path(), payload.into())
            .await
    }

    /// Fetch MFA methods.
    pub async fn fetch_mfa_methods(&self) -> KahoResult<MfaMethods> {
        self.get(Endpoint::MfaMethods.path()).await
    }

    /// Enable TOTP-based MFA.
    pub async fn enable_mfa_totp(
        &self,
        payload: impl Into<MfaTotpPayload>,
    ) -> KahoResult<MfaResponse> {
        self.put_return(Endpoint::MfaTotp.path(), payload.into())
            .await
    }

    /// Verify TOTP-based MFA.
    pub async fn verify_mfa_totp(
        &self,
        payload: impl Into<MfaTotpPayload>,
    ) -> KahoResult<MfaResponse> {
        self.post(Endpoint::MfaTotp.path(), payload.into()).await
    }

    /// Disable TOTP-based MFA.
    pub async fn disable_mfa_totp(&self, payload: impl Into<MfaTotpPayload>) -> KahoResult {
        self.delete(Endpoint::MfaTotp.path(), Some(payload.into()))
            .await
    }

    /// Fetch sync settings.
    pub async fn fetch_sync_settings(
        &self,
        payload: impl Into<SyncSettingsFetch>,
    ) -> KahoResult<SyncSettings> {
        self.post(Endpoint::SyncSettingsFetch.path(), payload.into())
            .await
    }

    /// Set a sync setting.
    pub async fn set_sync_setting(&self, payload: impl Into<SyncSettingsSet>) -> KahoResult {
        self.post_empty(Endpoint::SyncSettingsSet.path(), payload.into())
            .await
    }

    /// Fetch unread counts.
    pub async fn fetch_unreads(&self) -> KahoResult<Unreads> {
        self.get(Endpoint::SyncUnreads.path()).await
    }

    /// Subscribe to push notifications.
    pub async fn subscribe_push(&self, payload: impl Into<PushSubscription>) -> KahoResult {
        self.post_empty(Endpoint::PushSubscribe.path(), payload.into())
            .await
    }

    /// Unsubscribe from push notifications.
    pub async fn unsubscribe_push(&self, payload: impl Into<PushSubscription>) -> KahoResult {
        self.post_empty(Endpoint::PushUnsubscribe.path(), payload.into())
            .await
    }
}

fn duration_to_micros(duration: Duration) -> u64 {
    duration.as_micros().min(u64::MAX as u128) as u64
}
