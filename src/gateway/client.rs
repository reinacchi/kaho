use async_channel::{self, Receiver, Sender};
use futures::{pin_mut, SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::{select, spawn, time::sleep};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message},
};

use crate::{
    error::{KahoError, KahoResult},
    gateway::GatewayConfig,
    models::{ClientEvent, GatewayEvent},
};

/// Represents a gateway event stream value used by the Stoat API models and endpoints.
#[derive(Debug, Clone)]
pub struct GatewayEventStream {
    receiver: Receiver<KahoResult<GatewayEvent>>,
}

impl GatewayEventStream {
    /// Wait for the next gateway event.
    ///
    /// This method intentionally avoids the `StreamExt::next` `Unpin` bound,
    /// so callers do not need to use `std::pin`, `pin!`, or `Box::pin`.
    pub async fn next(&mut self) -> Option<KahoResult<GatewayEvent>> {
        self.receiver.recv().await.ok()
    }
}

/// WebSocket gateway client responsible for sending and receiving gateway events.
#[derive(Debug, Clone)]
pub struct GatewayClient {
    /// The config value associated with this gateway client.
    pub config: GatewayConfig,
    /// Timestamps for the most recent heartbeat ping and pong.
    pub last_heartbeat: (Instant, Instant),
    client_sender: Sender<ClientEvent>,
    client_receiver: Receiver<ClientEvent>,
    server_sender: Sender<Result<GatewayEvent, KahoError>>,
    server_receiver: Receiver<Result<GatewayEvent, KahoError>>,
    /// Whether the client believes it has started a gateway connection loop.
    pub is_connected: bool,
}

impl GatewayClient {
    /// Create a gateway client from an existing configuration.
    pub fn new(config: GatewayConfig) -> Self {
        let (client_sender, client_receiver) = async_channel::unbounded();
        let (server_sender, server_receiver) = async_channel::unbounded();

        Self {
            config,
            last_heartbeat: (Instant::now(), Instant::now()),
            client_receiver,
            client_sender,
            server_receiver,
            server_sender,
            is_connected: false,
        }
    }

    /// Start the gateway connection and reconnect loop.
    pub async fn connect(&mut self) -> KahoResult<()> {
        if self.is_connected {
            return Ok(());
        }

        let mut client = self.clone();
        spawn(async move {
            loop {
                match client.try_connect().await {
                    Ok(_) => {
                        client.config.reconnect_attempts = 0;
                    }
                    Err(e) => {
                        client.is_connected = false;
                        client.config.reconnect_attempts += 1;

                        if client.config.reconnect_attempts > client.config.max_reconnect_attempts {
                            let _ = client
                                .server_sender
                                .send(Err(KahoError::Other(format!(
                                    "Connection failed after {} reconnect attempts: {}",
                                    client.config.max_reconnect_attempts, e
                                ))))
                                .await;
                            break;
                        }

                        let delay = std::cmp::min(
                            client.config.reconnect_delay * client.config.reconnect_attempts as u32,
                            Duration::from_secs(60),
                        );

                        let _ = client
                            .server_sender
                            .send(Err(KahoError::Other(format!(
                                "Connection failed: {}; retrying in {}s",
                                e,
                                delay.as_secs()
                            ))))
                            .await;

                        sleep(delay).await;
                    }
                }
            }
        });

        self.is_connected = true;
        Ok(())
    }

    async fn try_connect(&mut self) -> KahoResult<()> {
        let (stream, _) = match connect_async(&self.config.ws_url).await {
            Ok((stream, response)) => (stream, response),
            Err(e) => {
                return Err(handle_websocket_error(e));
            }
        };

        self.is_connected = true;
        self.config.reconnect_attempts = 0;

        self.send(ClientEvent::Authenticate {
            token: self.config.token.clone(),
        })
        .await
        .map_err(|_e| KahoError::Other("Failed to send authentication event".into()))?;

        let client_receiver = self.client_receiver.clone();
        let server_sender = self.server_sender.clone();
        let heartbeat_sender = self.client_sender.clone();

        let heartbeat_task = spawn({
            let interval = self.config.heartbeat_interval;
            async move {
                let _ = Self::heartbeat(heartbeat_sender, interval).await;
            }
        });

        let (mut write_stream, mut read_stream) = stream.split();

        let write_task = spawn({
            let server_sender = server_sender.clone();
            async move {
                pin_mut!(client_receiver);

                while let Some(event) = client_receiver.next().await {
                    let msg = match serialize_client_event(&event) {
                        Ok(msg) => msg,
                        Err(e) => {
                            let _ = server_sender.send(Err(e)).await;
                            continue;
                        }
                    };

                    if let Err(e) = write_stream.send(msg).await {
                        let _ = server_sender
                            .send(Err(handle_websocket_error(e).into()))
                            .await;
                        break;
                    }
                }
            }
        });

        let read_task = spawn({
            let server_sender = server_sender.clone();
            async move {
                while let Some(msg) = read_stream.next().await {
                    let event = match msg {
                        Ok(msg) => match msg {
                            Message::Text(text) => deserialize_gateway_event_text(&text),
                            Message::Binary(_bytes) => {
                                #[cfg(feature = "msgpack")]
                                {
                                    deserialize_gateway_event_binary(&_bytes)
                                }
                                #[cfg(not(feature = "msgpack"))]
                                {
                                    continue;
                                }
                            }
                            Message::Close(_) => {
                                break;
                            }
                            _ => continue,
                        },
                        Err(e) => Err(handle_websocket_error(e).into()),
                    };

                    if matches!(event, Ok(GatewayEvent::Pong)) {
                        continue;
                    }

                    if server_sender.send(event).await.is_err() {
                        break;
                    }
                }

                let _ = server_sender
                    .send(Err(KahoError::Other("WebSocket disconnected".to_string())))
                    .await;
            }
        });

        select! {
            _ = heartbeat_task => Err(KahoError::Other("Heartbeat task terminated".into())),
            _ = write_task => Err(KahoError::Other("Write task terminated".into())),
            _ = read_task => Err(KahoError::Other("Read task terminated".into())),
        }
    }

    /// Queue a client event to be sent over the gateway connection.
    pub async fn send(&self, event: ClientEvent) -> KahoResult<()> {
        self.client_sender
            .send(event)
            .await
            .map_err(|e| KahoError::Other(format!("Failed to send event to client: {}", e)))
    }
    /// Returns a receiver-like gateway event stream.
    ///
    /// The returned type has an inherent async [`GatewayEventStream::next`]
    /// method, so consumers can write `events.next().await` without importing
    /// or using any pinning APIs.
    pub fn events(&self) -> GatewayEventStream {
        GatewayEventStream {
            receiver: self.server_receiver.clone(),
        }
    }

    /// Return the current heartbeat latency estimate.
    pub fn latency(&self) -> Duration {
        let (last_ping, last_pong) = self.last_heartbeat;
        if last_pong >= last_ping {
            last_pong.duration_since(last_ping)
        } else {
            last_ping.duration_since(last_pong)
        }
    }

    async fn heartbeat(sender: Sender<ClientEvent>, interval: Duration) -> Result<(), KahoError> {
        loop {
            if let Err(_e) = sender.send(ClientEvent::Ping { data: 0 }).await {
                break;
            }
            sleep(interval).await;
        }
        Ok(())
    }
}

fn handle_websocket_error(err: WsError) -> KahoError {
    match &err {
        WsError::AlreadyClosed => KahoError::Other("WebSocket already closed".to_string()),
        WsError::Io(io_err) if io_err.raw_os_error() == Some(104) => {
            KahoError::Other("Connection reset by peer".to_string())
        }
        WsError::Io(io_err) if io_err.raw_os_error() == Some(10054) => {
            KahoError::Other("Connection forcibly closed by remote host".to_string())
        }
        _ => KahoError::WebSocket(err),
    }
}

#[cfg(not(feature = "msgpack"))]
fn serialize_client_event(event: &ClientEvent) -> KahoResult<Message> {
    serde_json::to_string(event)
        .map(|json| Message::Text(json.into()))
        .map_err(|e| KahoError::Other(format!("Serialization error: {}", e)))
}

#[cfg(feature = "msgpack")]
fn serialize_client_event(event: &ClientEvent) -> KahoResult<Message> {
    rmp_serde::to_vec_named(event)
        .map(|bytes| Message::Binary(bytes.into()))
        .map_err(|e| KahoError::Other(format!("MessagePack serialization error: {}", e)))
}

fn deserialize_gateway_event_text(text: &str) -> KahoResult<GatewayEvent> {
    match serde_json::from_str::<GatewayEvent>(text) {
        Ok(GatewayEvent::Pong) => Ok(GatewayEvent::Pong),
        Ok(event) => Ok(event),
        Err(e) => Err(KahoError::Other(format!("Deserialization error: {}", e))),
    }
}

#[cfg(feature = "msgpack")]
fn deserialize_gateway_event_binary(bytes: &[u8]) -> KahoResult<GatewayEvent> {
    match rmp_serde::from_slice::<GatewayEvent>(bytes) {
        Ok(GatewayEvent::Pong) => Ok(GatewayEvent::Pong),
        Ok(event) => Ok(event),
        Err(e) => Err(KahoError::Other(format!("MessagePack deserialization error: {}", e))),
    }
}
