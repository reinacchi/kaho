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

/// Represents the main Kaho client.
#[derive(Clone, Debug)]
pub struct KahoClient {
    /// The HTTP client.
    pub http: HttpClient,
    /// The gateway client.
    pub gateway: GatewayClient,
    /// Shared in-memory model cache.
    #[cfg(feature = "cache")]
    pub cache: Cache,
}

impl KahoClient {
    /// Create a new client instance.
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
            #[cfg(feature = "cache")]
            cache: Cache::new(),
        })
    }
}
