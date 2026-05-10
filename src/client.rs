use crate::{
    error::{KahoError, KahoResult},
    gateway::{GatewayClient, GatewayConfig},
    http::{HttpClient, HttpConfig},
};

/// Represents the main Kaho client.
#[derive(Clone, Debug)]
pub struct KahoClient {
    /// The HTTP client.
    pub http: HttpClient,
    /// The gateway client.
    pub gateway: GatewayClient,
}

impl KahoClient {
    /// Create a new client instance.
    pub fn new(http: HttpClient, gateway: GatewayClient) -> Self {
        KahoClient { http, gateway }
    }

    /// Connect the bot to the gateway.
    pub async fn connect(&mut self) -> KahoResult<()> {
        self.gateway.connect().await
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
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// The bot token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Build a Kaho client.
    pub fn build(self) -> KahoResult<KahoClient> {
        let token = self
            .token
            .ok_or_else(|| KahoError::Other("Token must be provided".into()))?;

        let http_config = HttpConfig::new(&token)?;
        let gateway_config = GatewayConfig::new(&token)?;

        Ok(KahoClient {
            http: HttpClient::new(http_config)?,
            gateway: GatewayClient::new(gateway_config),
        })
    }
}
