use crate::error::KahoError;

#[derive(Clone, Debug)]
pub struct HttpConfig {
    pub token: String,
    pub api_url: String,
}

impl HttpConfig {
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
