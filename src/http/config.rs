use crate::error::KahoError;

/// Configuration used by the HTTP client.
#[derive(Clone, Debug)]
pub struct HttpConfig {
    /// Bot token used for API authentication.
    pub token: String,
    /// Base URL of the Stoat HTTP API.
    pub api_url: String,
}

impl HttpConfig {
    /// Create HTTP configuration for a bot token.
    pub fn new(token: impl Into<String>) -> Result<Self, KahoError> {
        let token = token.into();
        if token.is_empty() {
            return Err(KahoError::Other("Token cannot be empty".into()));
        }
        Ok(HttpConfig {
            token,
            api_url: "https://stoat.chat/api".into(),
        })
    }
}
