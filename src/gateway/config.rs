use std::time::Duration;

use crate::error::KahoError;

/// Configuration used to connect and maintain the gateway WebSocket.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    /// The heartbeat interval value associated with this gateway config.
    pub heartbeat_interval: Duration,
    /// Maximum number of reconnect attempts before the gateway gives up.
    pub max_reconnect_attempts: usize,
    /// Number of reconnect attempts made by the current connection loop.
    pub reconnect_attempts: usize,
    /// The reconnect delay value associated with this gateway config.
    pub reconnect_delay: Duration,
    /// Bot token used for gateway authentication.
    pub token: String,
    /// WebSocket URL used for gateway connections.
    pub ws_url: String,
}

impl GatewayConfig {
    /// Calls the Stoat API or client internals to new for this resource.
    pub fn new(token: impl Into<String>) -> Result<Self, KahoError> {
        let token = token.into();

        if token.is_empty() {
            return Err(KahoError::Other("Token cannot be empty".into()));
        }

        let ws_url = if cfg!(feature = "msgpack") {
            "wss://ws.stoat.chat/events?format=msgpack"
        } else {
            "wss://stoat.chat/events"
        };

        Ok(GatewayConfig {
            heartbeat_interval: Duration::from_secs(15),
            max_reconnect_attempts: 5,
            reconnect_attempts: 0,
            reconnect_delay: Duration::from_secs(5),
            token,
            ws_url: ws_url.into(),
        })
    }
}
