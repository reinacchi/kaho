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
}

impl HttpConfig {
    /// Create a new instance using the public Stoat endpoints.
    pub fn new(token: impl Into<String>) -> Result<Self, KahoError> {
        let token = token.into();
        if token.is_empty() {
            return Err(KahoError::Other("Token cannot be empty".into()));
        }

        Ok(HttpConfig {
            token,
            api_url: "https://stoat.chat/api".into(),
            cdn_url: "https://cdn.stoatusercontent.com".into(),
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
}
