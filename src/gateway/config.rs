use std::time::Duration;

use crate::error::KahoError;

const DEFAULT_GATEWAY_URL: &str = "wss://stoat.chat/events";
const DEFAULT_READY_FIELDS: &[&str] = &["users", "servers", "channels", "members"];

/// Configuration used to connect and maintain the gateway WebSocket.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    /// Maximum time allowed for establishing the WebSocket connection.
    pub connect_timeout: Duration,
    /// Maximum time allowed for gateway authentication to complete.
    pub authentication_timeout: Duration,
    /// The heartbeat interval value associated with this gateway config.
    pub heartbeat_interval: Duration,
    /// Maximum time Kaho waits for the Pong corresponding to a Ping.
    pub heartbeat_timeout: Duration,
    /// Maximum number of consecutive reconnect attempts before the gateway gives up.
    ///
    /// A value of `0` means retry indefinitely, which is the default for long-running bots.
    pub max_reconnect_attempts: usize,
    /// Maximum delay between reconnect attempts.
    pub max_reconnect_delay: Duration,
    /// Number of reconnect attempts made by the current connection loop.
    ///
    /// This field is retained for configuration compatibility. Runtime reconnect counters are
    /// exposed through [`crate::gateway::GatewayClient::metrics`].
    pub reconnect_attempts: usize,
    /// Base reconnect delay. Kaho applies exponential backoff and jitter on top of this value.
    pub reconnect_delay: Duration,
    /// Maximum time allowed for writing a packet to the WebSocket.
    pub write_timeout: Duration,
    /// Maximum time a public client event may wait for space in the outbound queue.
    pub outbound_queue_timeout: Duration,
    /// Maximum number of client-to-gateway events buffered while writers are busy.
    pub outbound_queue_capacity: usize,
    /// Maximum number of gateway events buffered for the application.
    ///
    /// When this queue is full Kaho drops the oldest application event, while still applying the
    /// incoming event to the cache first. Queue overflows are visible in gateway metrics.
    pub event_queue_capacity: usize,
    /// Bot token used for gateway authentication.
    pub token: String,
    /// WebSocket URL used for gateway connections.
    pub ws_url: String,
}

impl GatewayConfig {
    /// Create a new instance using Stoat's current gateway protocol.
    pub fn new(token: impl Into<String>) -> Result<Self, KahoError> {
        let token = token.into();

        if token.is_empty() {
            return Err(KahoError::Other("Token cannot be empty".into()));
        }

        Ok(Self {
            connect_timeout: Duration::from_secs(15),
            authentication_timeout: Duration::from_secs(15),
            heartbeat_interval: Duration::from_secs(15),
            heartbeat_timeout: Duration::from_secs(30),
            max_reconnect_attempts: 0,
            max_reconnect_delay: Duration::from_secs(60),
            reconnect_attempts: 0,
            reconnect_delay: Duration::from_secs(2),
            write_timeout: Duration::from_secs(15),
            outbound_queue_timeout: Duration::from_secs(5),
            outbound_queue_capacity: 1_024,
            event_queue_capacity: 4_096,
            token,
            ws_url: default_gateway_url(),
        })
    }

    /// Override the gateway WebSocket URL.
    pub fn with_ws_url(mut self, ws_url: impl Into<String>) -> Self {
        self.ws_url = ws_url.into();
        self
    }

    /// Override the maximum number of consecutive reconnect attempts.
    ///
    /// Set this to `0` to retry indefinitely.
    pub fn with_max_reconnect_attempts(mut self, attempts: usize) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }

    /// Override the base and maximum reconnect delays.
    pub fn with_reconnect_backoff(mut self, base: Duration, maximum: Duration) -> Self {
        self.reconnect_delay = base;
        self.max_reconnect_delay = maximum.max(base);
        self
    }

    /// Override heartbeat timings.
    pub fn with_heartbeat(mut self, interval: Duration, timeout: Duration) -> Self {
        self.heartbeat_interval = interval;
        self.heartbeat_timeout = timeout.max(interval);
        self
    }

    /// Override gateway queue capacities.
    pub fn with_queue_capacities(mut self, outbound: usize, events: usize) -> Self {
        self.outbound_queue_capacity = outbound.max(1);
        self.event_queue_capacity = events.max(1);
        self
    }
}

fn default_gateway_url() -> String {
    let format = if cfg!(feature = "msgpack") {
        "msgpack"
    } else {
        "json"
    };

    let mut url = format!("{DEFAULT_GATEWAY_URL}?version=1&format={format}");
    for field in DEFAULT_READY_FIELDS {
        url.push_str("&ready=");
        url.push_str(field);
    }
    url
}

#[cfg(test)]
mod tests {
    use super::GatewayConfig;

    #[test]
    fn default_url_uses_current_stoat_gateway_protocol() {
        let config = GatewayConfig::new("token").expect("valid config");

        assert!(config.ws_url.starts_with("wss://stoat.chat/events?"));
        assert!(config.ws_url.contains("version=1"));
        assert!(config.ws_url.contains("ready=servers"));
        assert!(config.ws_url.contains("ready=members"));
        assert!(!config.ws_url.contains("ready=emojis"));
    }

    #[test]
    fn reconnects_are_unlimited_by_default() {
        let config = GatewayConfig::new("token").expect("valid config");
        assert_eq!(config.max_reconnect_attempts, 0);
    }
}
