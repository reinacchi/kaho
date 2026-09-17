use std::time::Duration;

use crate::error::KahoError;

/// Configuration used by the HTTP client.
#[derive(Clone, Debug)]
pub struct HttpConfig {
    /// Bot token used for API authentication.
    pub token: String,
    /// Base URL of the Stoat HTTP API.
    pub api_url: String,
    /// Base URL of the Stoat file CDN/upload service.
    pub cdn_url: String,
    /// Maximum time allowed for establishing a new TCP/TLS connection.
    pub connect_timeout: Duration,
    /// Maximum time allowed for a complete HTTP request.
    pub request_timeout: Duration,
    /// How long idle pooled connections are kept alive for reuse.
    pub pool_idle_timeout: Duration,
    /// TCP keepalive interval used for long-lived pooled connections.
    pub tcp_keepalive: Duration,
}

impl HttpConfig {
    /// Create a new instance using the public Stoat endpoints.
    pub fn new(token: impl Into<String>) -> Result<Self, KahoError> {
        let token = token.into();
        if token.is_empty() {
            return Err(KahoError::Other("Token cannot be empty".into()));
        }

        Ok(Self {
            token,
            api_url: "https://stoat.chat/api".into(),
            cdn_url: "https://cdn.stoatusercontent.com".into(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            pool_idle_timeout: Duration::from_secs(90),
            tcp_keepalive: Duration::from_secs(30),
        })
    }

    /// Override the API base URL.
    pub fn with_api_url(mut self, api_url: impl Into<String>) -> Self {
        self.api_url = api_url.into();
        self
    }

    /// Override the CDN base URL used for file uploads.
    pub fn with_cdn_url(mut self, cdn_url: impl Into<String>) -> Self {
        self.cdn_url = cdn_url.into();
        self
    }

    /// Override HTTP connection and whole-request timeouts.
    pub fn with_timeouts(mut self, connect: Duration, request: Duration) -> Self {
        self.connect_timeout = connect;
        self.request_timeout = request;
        self
    }
}
