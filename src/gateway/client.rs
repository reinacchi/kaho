use async_channel::{bounded, Receiver, Sender, TryRecvError, TrySendError};
use futures::{Sink, SinkExt, Stream, StreamExt};
#[cfg(feature = "msgpack")]
use rmp_serde::{from_slice as from_msgpack_slice, to_vec_named as to_msgpack_vec};
use serde_json::{from_str as from_json_str, to_string as to_json_string};
use std::{
    cmp::min,
    future::pending,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    select, spawn,
    sync::watch,
    time::{interval, sleep, timeout, MissedTickBehavior},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message},
};
use tracing::{debug, warn};

#[cfg(feature = "cache")]
use crate::cache::Cache;
use crate::{
    error::{AuthError, KahoError, KahoResult},
    gateway::GatewayConfig,
    models::{ClientEvent, GatewayEvent},
};

/// Current lifecycle state of the gateway connection loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum GatewayConnectionState {
    /// No connection loop is currently running.
    Disconnected = 0,
    /// Kaho is establishing or authenticating the first connection.
    Connecting = 1,
    /// The gateway has authenticated the current connection.
    Connected = 2,
    /// Kaho is waiting before or attempting a reconnect.
    Reconnecting = 3,
    /// The connection loop stopped after a fatal error or configured retry limit.
    Stopped = 4,
}

impl GatewayConnectionState {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::Reconnecting,
            4 => Self::Stopped,
            _ => Self::Disconnected,
        }
    }
}

/// Point-in-time gateway diagnostics useful for identifying latency and reconnect issues.
#[derive(Clone, Debug)]
pub struct GatewayMetrics {
    /// Current gateway lifecycle state.
    pub connection_state: GatewayConnectionState,
    /// Number of reconnects attempted since this gateway client was created.
    pub reconnects: u64,
    /// Number of decoded gateway events received, including heartbeat events.
    pub events_received: u64,
    /// Number of application events discarded because the bounded event queue overflowed.
    pub dropped_events: u64,
    /// Number of application events currently waiting to be consumed.
    pub event_queue_depth: usize,
    /// Maximum number of application events buffered by the gateway client.
    pub event_queue_capacity: usize,
    /// Number of client events currently waiting to be written to the gateway.
    pub outbound_queue_depth: usize,
    /// Maximum number of client events buffered while the gateway writer is busy.
    pub outbound_queue_capacity: usize,
    /// Delay experienced by the most recently consumed application event.
    pub last_event_queue_delay: Duration,
    /// Time elapsed since the most recent decoded gateway event.
    pub time_since_last_event: Option<Duration>,
    /// Most recently measured Stoat Ping/Pong round-trip latency.
    pub heartbeat_latency: Duration,
}

#[derive(Debug)]
struct GatewayMetricsInner {
    state: AtomicU8,
    reconnects: AtomicU64,
    events_received: AtomicU64,
    dropped_events: AtomicU64,
    last_queue_delay_micros: AtomicU64,
    last_event_at: Mutex<Option<Instant>>,
}

impl Default for GatewayMetricsInner {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(GatewayConnectionState::Disconnected as u8),
            reconnects: AtomicU64::new(0),
            events_received: AtomicU64::new(0),
            dropped_events: AtomicU64::new(0),
            last_queue_delay_micros: AtomicU64::new(0),
            last_event_at: Mutex::new(None),
        }
    }
}

#[derive(Debug, Default)]
struct GatewayShared {
    loop_started: AtomicBool,
    heartbeat_nonce: AtomicUsize,
    awaiting_pong: AtomicBool,
    metrics: GatewayMetricsInner,
}

#[derive(Debug)]
struct QueuedGatewayEvent {
    enqueued_at: Instant,
    result: KahoResult<GatewayEvent>,
}

/// Represents a gateway event stream value used by the Stoat API models and endpoints.
#[derive(Debug, Clone)]
pub struct GatewayEventStream {
    receiver: Receiver<QueuedGatewayEvent>,
    shared: Arc<GatewayShared>,
    observed_dropped_events: u64,
}

impl GatewayEventStream {
    /// Wait for the next gateway event.
    pub async fn next(&mut self) -> Option<KahoResult<GatewayEvent>> {
        let dropped_events = self.shared.metrics.dropped_events.load(Ordering::Relaxed);
        if dropped_events > self.observed_dropped_events {
            let dropped = dropped_events - self.observed_dropped_events;
            self.observed_dropped_events = dropped_events;
            return Some(Err(KahoError::GatewayEventQueueOverflow { dropped }));
        }

        let queued = self.receiver.recv().await.ok()?;
        self.shared.metrics.last_queue_delay_micros.store(
            duration_to_micros(queued.enqueued_at.elapsed()),
            Ordering::Relaxed,
        );
        Some(queued.result)
    }
}

/// WebSocket gateway client responsible for sending and receiving gateway events.
#[derive(Debug, Clone)]
pub struct GatewayClient {
    /// The config value associated with this gateway client.
    pub config: GatewayConfig,
    /// Timestamps for the most recent heartbeat ping and pong.
    pub last_heartbeat: Arc<Mutex<(Option<Instant>, Option<Instant>)>>,
    client_sender: Sender<ClientEvent>,
    client_receiver: Receiver<ClientEvent>,
    server_sender: Sender<QueuedGatewayEvent>,
    server_receiver: Receiver<QueuedGatewayEvent>,
    shutdown_sender: watch::Sender<bool>,
    shutdown_receiver: watch::Receiver<bool>,
    shared: Arc<GatewayShared>,
    #[cfg(feature = "cache")]
    cache: Option<Cache>,
}

impl GatewayClient {
    /// Create a gateway client from an existing configuration.
    pub fn new(config: GatewayConfig) -> Self {
        let (client_sender, client_receiver) = bounded(config.outbound_queue_capacity.max(1));
        let (server_sender, server_receiver) = bounded(config.event_queue_capacity.max(1));
        let (shutdown_sender, shutdown_receiver) = watch::channel(false);

        Self {
            config,
            last_heartbeat: Arc::new(Mutex::new((None, None))),
            client_receiver,
            client_sender,
            server_receiver,
            server_sender,
            shutdown_sender,
            shutdown_receiver,
            shared: Arc::new(GatewayShared::default()),
            #[cfg(feature = "cache")]
            cache: None,
        }
    }

    /// Attach the shared Kaho cache so gateway events update it as soon as they arrive.
    #[cfg(feature = "cache")]
    pub(crate) fn set_cache(&mut self, cache: Cache) {
        self.cache = Some(cache);
    }

    /// Start the gateway connection and reconnect loop.
    ///
    /// Calling this method more than once while the loop is active is a no-op.
    pub async fn connect(&self) -> KahoResult<()> {
        if self.shared.loop_started.swap(true, Ordering::AcqRel) {
            return Ok(());
        }

        let _ = self.shutdown_sender.send(false);
        self.set_connection_state(GatewayConnectionState::Connecting);

        let client = self.clone();
        spawn(async move {
            client.run_connection_loop().await;
        });

        Ok(())
    }

    /// Request a clean stop of the active connection and reconnect loop.
    ///
    /// A later call to [`GatewayClient::connect`] can start it again.
    pub fn disconnect(&self) {
        let _ = self.shutdown_sender.send(true);
    }

    async fn run_connection_loop(self) {
        let mut consecutive_failures = 0usize;
        let mut first_attempt = true;

        loop {
            if *self.shutdown_receiver.borrow() {
                self.set_connection_state(GatewayConnectionState::Disconnected);
                break;
            }

            self.set_connection_state(if first_attempt {
                GatewayConnectionState::Connecting
            } else {
                GatewayConnectionState::Reconnecting
            });

            let session = self.run_session().await;
            self.shared.awaiting_pong.store(false, Ordering::Release);

            if *self.shutdown_receiver.borrow() {
                self.set_connection_state(GatewayConnectionState::Disconnected);
                break;
            }

            if session.authenticated {
                consecutive_failures = 0;
            }

            if is_fatal_gateway_error(&session.error) {
                self.enqueue_result(Err(session.error));
                self.set_connection_state(GatewayConnectionState::Stopped);
                break;
            }

            consecutive_failures = consecutive_failures.saturating_add(1);
            self.shared
                .metrics
                .reconnects
                .fetch_add(1, Ordering::Relaxed);

            if self.config.max_reconnect_attempts != 0
                && consecutive_failures > self.config.max_reconnect_attempts
            {
                self.enqueue_result(Err(KahoError::Other(format!(
                    "Gateway stopped after {} consecutive reconnect attempts: {}",
                    self.config.max_reconnect_attempts, session.error
                ))));
                self.set_connection_state(GatewayConnectionState::Stopped);
                break;
            }

            let delay = reconnect_delay(&self.config, consecutive_failures);
            warn!(
                error = %session.error,
                reconnect_in_ms = delay.as_millis(),
                attempt = consecutive_failures,
                "gateway disconnected. reconnecting"
            );
            self.enqueue_result(Err(session.error));
            self.set_connection_state(GatewayConnectionState::Reconnecting);
            first_attempt = false;

            select! {
                _ = sleep(delay) => {}
                _ = wait_for_shutdown(self.shutdown_receiver.clone()) => {
                    self.set_connection_state(GatewayConnectionState::Disconnected);
                    break;
                }
            }
        }

        self.shared.loop_started.store(false, Ordering::Release);
    }

    async fn run_session(&self) -> SessionEnd {
        let connect = match timeout(
            self.config.connect_timeout,
            connect_async(&self.config.ws_url),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                return SessionEnd {
                    error: KahoError::GatewayConnectTimeout,
                    authenticated: false,
                };
            }
        };

        let (mut stream, _response) = match connect {
            Ok(value) => value,
            Err(error) => {
                return SessionEnd {
                    error: handle_websocket_error(error),
                    authenticated: false,
                };
            }
        };

        if let Ok(mut heartbeat) = self.last_heartbeat.lock() {
            *heartbeat = (None, None);
        }
        self.shared.awaiting_pong.store(false, Ordering::Release);

        let authentication = ClientEvent::Authenticate {
            token: self.config.token.clone(),
        };
        let authentication = match serialize_client_event(&authentication) {
            Ok(message) => message,
            Err(error) => {
                return SessionEnd {
                    error,
                    authenticated: false,
                };
            }
        };

        match timeout(self.config.write_timeout, stream.send(authentication)).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                return SessionEnd {
                    error: handle_websocket_error(error),
                    authenticated: false,
                };
            }
            Err(_) => {
                return SessionEnd {
                    error: KahoError::GatewayWriteTimeout,
                    authenticated: false,
                };
            }
        }

        let authenticated = AtomicBool::new(false);
        let (write_stream, read_stream) = stream.split();

        let read_loop = self.read_loop(read_stream, &authenticated);
        let write_loop = self.write_loop(write_stream);
        let auth_watchdog =
            authentication_watchdog(&authenticated, self.config.authentication_timeout);
        let heartbeat_watchdog = self.heartbeat_watchdog();

        let result = select! {
            result = read_loop => result,
            result = write_loop => result,
            result = auth_watchdog => result,
            result = heartbeat_watchdog => result,
            _ = wait_for_shutdown(self.shutdown_receiver.clone()) => {
                Err(KahoError::Other("Gateway shutdown requested".into()))
            }
        };

        SessionEnd {
            error: result.err().unwrap_or_else(|| {
                KahoError::Other("Gateway session terminated unexpectedly".into())
            }),
            authenticated: authenticated.load(Ordering::Acquire),
        }
    }

    async fn read_loop<S>(&self, mut read_stream: S, authenticated: &AtomicBool) -> KahoResult
    where
        S: Stream<Item = Result<Message, WsError>> + Unpin,
    {
        while let Some(message) = read_stream.next().await {
            let event = match message {
                Ok(Message::Text(text)) => deserialize_gateway_event_text(&text),
                Ok(Message::Binary(bytes)) => {
                    #[cfg(feature = "msgpack")]
                    {
                        deserialize_gateway_event_binary(&bytes)
                    }
                    #[cfg(not(feature = "msgpack"))]
                    {
                        let _ = bytes;
                        continue;
                    }
                }
                Ok(Message::Close(frame)) => {
                    let reason = frame
                        .map(|frame| frame.reason.to_string())
                        .filter(|reason| !reason.is_empty())
                        .unwrap_or_else(|| "remote endpoint closed the WebSocket".into());
                    return Err(KahoError::Other(reason));
                }
                Ok(_) => continue,
                Err(error) => return Err(handle_websocket_error(error)),
            };

            match event {
                Ok(event) => self.process_gateway_event(event, authenticated).await?,
                Err(error) => {
                    debug!(%error, "discarding malformed gateway event");
                    self.enqueue_result(Err(error));
                }
            }
        }

        Err(KahoError::Other("WebSocket disconnected".into()))
    }

    async fn write_loop<S>(&self, mut write_stream: S) -> KahoResult
    where
        S: Sink<Message, Error = WsError> + Unpin,
    {
        let heartbeat_interval = self
            .config
            .heartbeat_interval
            .max(Duration::from_millis(100));
        let mut heartbeat = interval(heartbeat_interval);
        heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
        // Tokio intervals tick immediately once. Consume that initial tick so the first gateway
        // Ping is sent after the configured interval rather than immediately after Authenticate.
        heartbeat.tick().await;

        loop {
            select! {
                event = self.client_receiver.recv() => {
                    let event = event.map_err(|error| {
                        KahoError::Other(format!("Gateway outbound queue closed: {error}"))
                    })?;
                    let message = serialize_client_event(&event)?;
                    send_websocket_message(
                        &mut write_stream,
                        message,
                        self.config.write_timeout,
                    ).await?;
                }
                _ = heartbeat.tick() => {
                    self.send_heartbeat(&mut write_stream).await?;
                }
            }
        }
    }

    async fn heartbeat_watchdog(&self) -> KahoResult {
        let check_interval = min(
            Duration::from_secs(1),
            self.config
                .heartbeat_timeout
                .max(Duration::from_millis(100)),
        );

        loop {
            sleep(check_interval).await;
            if !self.shared.awaiting_pong.load(Ordering::Acquire) {
                continue;
            }

            let timed_out = self
                .last_heartbeat
                .lock()
                .ok()
                .and_then(|heartbeat| heartbeat.0)
                .map(|ping| ping.elapsed() >= self.config.heartbeat_timeout)
                .unwrap_or(false);
            if timed_out {
                return Err(KahoError::GatewayHeartbeatTimeout);
            }
        }
    }

    async fn send_heartbeat<S>(&self, write_stream: &mut S) -> KahoResult
    where
        S: Sink<Message, Error = WsError> + Unpin,
    {
        if self.shared.awaiting_pong.load(Ordering::Acquire) {
            let ping_age = self
                .last_heartbeat
                .lock()
                .ok()
                .and_then(|heartbeat| heartbeat.0)
                .map(|ping| ping.elapsed());

            if ping_age
                .map(|age| age >= self.config.heartbeat_timeout)
                .unwrap_or(false)
            {
                return Err(KahoError::GatewayHeartbeatTimeout);
            }

            // Do not overwrite the timestamp for an outstanding ping. This ensures a missing
            // Pong eventually trips the watchdog instead of being hidden by newer pings.
            return Ok(());
        }

        let data = self
            .shared
            .heartbeat_nonce
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        let message = serialize_client_event(&ClientEvent::Ping { data })?;
        let sent_at = Instant::now();

        if let Ok(mut heartbeat) = self.last_heartbeat.lock() {
            heartbeat.0 = Some(sent_at);
        }
        self.shared.awaiting_pong.store(true, Ordering::Release);

        if let Err(error) =
            send_websocket_message(write_stream, message, self.config.write_timeout).await
        {
            self.shared.awaiting_pong.store(false, Ordering::Release);
            return Err(error);
        }

        Ok(())
    }

    async fn process_gateway_event(
        &self,
        event: GatewayEvent,
        authenticated: &AtomicBool,
    ) -> KahoResult {
        self.record_event_received();

        let mut v = match event {
            GatewayEvent::Bulk { mut v } => {
                v.reverse();
                v
            }
            event => return self.process_gateway_event_item(event, authenticated).await,
        };

        while let Some(event) = v.pop() {
            self.record_event_received();
            match event {
                GatewayEvent::Bulk { v: mut nested } => {
                    nested.reverse();
                    v.extend(nested);
                }
                event => {
                    self.process_gateway_event_item(event, authenticated)
                        .await?
                }
            }
        }

        Ok(())
    }

    fn record_event_received(&self) {
        self.shared
            .metrics
            .events_received
            .fetch_add(1, Ordering::Relaxed);
        if let Ok(mut last_event) = self.shared.metrics.last_event_at.lock() {
            *last_event = Some(Instant::now());
        }
    }

    async fn process_gateway_event_item(
        &self,
        event: GatewayEvent,
        authenticated: &AtomicBool,
    ) -> KahoResult {
        match event {
            GatewayEvent::Pong { data } => self.record_pong(data),
            GatewayEvent::Error { error } => return Err(KahoError::Auth(error)),
            GatewayEvent::LoggedOut => return Err(KahoError::GatewayLoggedOut),
            GatewayEvent::Bulk { .. } => unreachable!("bulk events are flattened before handling"),
            event => {
                if matches!(&event, GatewayEvent::Authenticated | GatewayEvent::Ready(_)) {
                    authenticated.store(true, Ordering::Release);
                    self.set_connection_state(GatewayConnectionState::Connected);
                }

                #[cfg(feature = "cache")]
                if let Some(cache) = &self.cache {
                    cache.update_from_event(&event).await;
                }

                self.enqueue_result(Ok(event));
            }
        }

        Ok(())
    }

    fn record_pong(&self, data: usize) {
        if !self.shared.awaiting_pong.load(Ordering::Acquire) {
            return;
        }

        if data != self.shared.heartbeat_nonce.load(Ordering::Relaxed) {
            return;
        }

        if let Ok(mut heartbeat) = self.last_heartbeat.lock() {
            heartbeat.1 = Some(Instant::now());
        }
        self.shared.awaiting_pong.store(false, Ordering::Release);
    }

    fn enqueue_result(&self, result: KahoResult<GatewayEvent>) {
        let mut queued = QueuedGatewayEvent {
            enqueued_at: Instant::now(),
            result,
        };

        loop {
            match self.server_sender.try_send(queued) {
                Ok(()) => return,
                Err(TrySendError::Closed(_)) => return,
                Err(TrySendError::Full(item)) => {
                    queued = item;
                    match self.server_receiver.try_recv() {
                        Ok(_) => {
                            let dropped = self
                                .shared
                                .metrics
                                .dropped_events
                                .fetch_add(1, Ordering::Relaxed)
                                .saturating_add(1);
                            if dropped == 1 || dropped.is_power_of_two() {
                                warn!(
                                    dropped_events = dropped,
                                    queue_capacity = self.server_sender.capacity().unwrap_or(0),
                                    "gateway application event queue overflowed; dropped oldest event"
                                );
                            }
                        }
                        Err(TryRecvError::Empty) => continue,
                        Err(TryRecvError::Closed) => return,
                    }
                }
            }
        }
    }

    fn set_connection_state(&self, state: GatewayConnectionState) {
        self.shared
            .metrics
            .state
            .store(state as u8, Ordering::Release);
    }

    /// Queue a client event to be sent over the gateway connection.
    pub async fn send(&self, event: ClientEvent) -> KahoResult<()> {
        match timeout(
            self.config.outbound_queue_timeout,
            self.client_sender.send(event),
        )
        .await
        {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(KahoError::Other(format!(
                "Failed to queue gateway event: {error}"
            ))),
            Err(_) => Err(KahoError::GatewaySendQueueTimeout),
        }
    }

    /// Returns a receiver-like gateway event stream.
    ///
    /// The returned type has an inherent async [`GatewayEventStream::next`] method, so consumers
    /// can write `events.next().await` without importing or using any pinning APIs.
    pub fn events(&self) -> GatewayEventStream {
        GatewayEventStream {
            receiver: self.server_receiver.clone(),
            shared: self.shared.clone(),
            observed_dropped_events: self.shared.metrics.dropped_events.load(Ordering::Relaxed),
        }
    }

    /// Return the current gateway connection state.
    pub fn connection_state(&self) -> GatewayConnectionState {
        GatewayConnectionState::from_u8(self.shared.metrics.state.load(Ordering::Acquire))
    }

    /// Return whether the current gateway session has authenticated successfully.
    pub fn is_connected(&self) -> bool {
        self.connection_state() == GatewayConnectionState::Connected
    }

    /// Return the current heartbeat latency estimate.
    ///
    /// Returns `Duration::ZERO` until at least one Ping/Pong cycle has completed.
    pub fn latency(&self) -> Duration {
        let Ok((last_ping, last_pong)) = self.last_heartbeat.lock().map(|state| *state) else {
            return Duration::ZERO;
        };

        match (last_ping, last_pong) {
            (Some(ping), Some(pong)) if pong >= ping => pong.duration_since(ping),
            _ => Duration::ZERO,
        }
    }

    /// Return a point-in-time gateway diagnostics snapshot.
    pub fn metrics(&self) -> GatewayMetrics {
        let time_since_last_event = self
            .shared
            .metrics
            .last_event_at
            .lock()
            .ok()
            .and_then(|last_event| last_event.map(|instant| instant.elapsed()));

        GatewayMetrics {
            connection_state: self.connection_state(),
            reconnects: self.shared.metrics.reconnects.load(Ordering::Relaxed),
            events_received: self.shared.metrics.events_received.load(Ordering::Relaxed),
            dropped_events: self.shared.metrics.dropped_events.load(Ordering::Relaxed),
            event_queue_depth: self.server_sender.len(),
            event_queue_capacity: self.server_sender.capacity().unwrap_or(0),
            outbound_queue_depth: self.client_sender.len(),
            outbound_queue_capacity: self.client_sender.capacity().unwrap_or(0),
            last_event_queue_delay: Duration::from_micros(
                self.shared
                    .metrics
                    .last_queue_delay_micros
                    .load(Ordering::Relaxed),
            ),
            time_since_last_event,
            heartbeat_latency: self.latency(),
        }
    }
}

#[derive(Debug)]
struct SessionEnd {
    error: KahoError,
    authenticated: bool,
}

async fn wait_for_shutdown(mut receiver: watch::Receiver<bool>) {
    if *receiver.borrow() {
        return;
    }

    loop {
        if receiver.changed().await.is_err() || *receiver.borrow() {
            return;
        }
    }
}

async fn authentication_watchdog(
    authenticated: &AtomicBool,
    authentication_timeout: Duration,
) -> KahoResult {
    sleep(authentication_timeout.max(Duration::from_millis(100))).await;

    if authenticated.load(Ordering::Acquire) {
        pending::<KahoResult>().await
    } else {
        Err(KahoError::GatewayAuthenticationTimeout)
    }
}

async fn send_websocket_message<S>(
    write_stream: &mut S,
    message: Message,
    write_timeout: Duration,
) -> KahoResult
where
    S: Sink<Message, Error = WsError> + Unpin,
{
    match timeout(
        write_timeout.max(Duration::from_millis(100)),
        write_stream.send(message),
    )
    .await
    {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(handle_websocket_error(error)),
        Err(_) => Err(KahoError::GatewayWriteTimeout),
    }
}

fn reconnect_delay(config: &GatewayConfig, attempt: usize) -> Duration {
    let exponent = attempt.saturating_sub(1).min(10) as u32;
    let multiplier = 1u32 << exponent;
    let base = config
        .reconnect_delay
        .checked_mul(multiplier)
        .unwrap_or(config.max_reconnect_delay);
    let capped = min(base, config.max_reconnect_delay);

    let maximum_jitter_ms = (capped.as_millis() / 5).min(u64::MAX as u128) as u64;
    if maximum_jitter_ms == 0 || capped >= config.max_reconnect_delay {
        return capped;
    }

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as u64)
        .unwrap_or(0);
    let jitter = Duration::from_millis(seed % (maximum_jitter_ms + 1));
    min(capped + jitter, config.max_reconnect_delay)
}

fn is_fatal_gateway_error(error: &KahoError) -> bool {
    matches!(
        error,
        KahoError::GatewayLoggedOut
            | KahoError::Auth(AuthError::InvalidSession)
            | KahoError::Auth(AuthError::OnboardingNotFinished)
            | KahoError::Auth(AuthError::AlreadyAuthenticated)
    )
}

fn handle_websocket_error(error: WsError) -> KahoError {
    match &error {
        WsError::AlreadyClosed => KahoError::Other("WebSocket already closed".to_string()),
        WsError::Io(io_error) if io_error.raw_os_error() == Some(104) => {
            KahoError::Other("Connection reset by peer".to_string())
        }
        WsError::Io(io_error) if io_error.raw_os_error() == Some(10054) => {
            KahoError::Other("Connection forcibly closed by remote host".to_string())
        }
        _ => KahoError::WebSocket(error),
    }
}

#[cfg(not(feature = "msgpack"))]
fn serialize_client_event(event: &ClientEvent) -> KahoResult<Message> {
    to_json_string(event)
        .map(|json| Message::Text(json.into()))
        .map_err(|error| KahoError::Other(format!("Serialization error: {error}")))
}

#[cfg(feature = "msgpack")]
fn serialize_client_event(event: &ClientEvent) -> KahoResult<Message> {
    to_msgpack_vec(event)
        .map(|bytes| Message::Binary(bytes.into()))
        .map_err(|error| KahoError::Other(format!("MessagePack serialization error: {error}")))
}

fn deserialize_gateway_event_text(text: &str) -> KahoResult<GatewayEvent> {
    from_json_str::<GatewayEvent>(text)
        .map_err(|error| KahoError::Other(format!("Deserialization error: {error}")))
}

#[cfg(feature = "msgpack")]
fn deserialize_gateway_event_binary(bytes: &[u8]) -> KahoResult<GatewayEvent> {
    from_msgpack_slice::<GatewayEvent>(bytes)
        .map_err(|error| KahoError::Other(format!("MessagePack deserialization error: {error}")))
}

fn duration_to_micros(duration: Duration) -> u64 {
    duration.as_micros().min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{reconnect_delay, GatewayConnectionState};
    use crate::{error::KahoError, gateway::GatewayConfig, models::GatewayEvent};

    #[test]
    fn reconnect_backoff_is_capped() {
        let mut config = GatewayConfig::new("token").expect("valid config");
        config.reconnect_delay = Duration::from_secs(2);
        config.max_reconnect_delay = Duration::from_secs(8);

        assert!(reconnect_delay(&config, 1) >= Duration::from_secs(2));
        assert!(reconnect_delay(&config, 10) <= Duration::from_secs(8));
    }

    #[test]
    fn connection_state_round_trips() {
        for state in [
            GatewayConnectionState::Disconnected,
            GatewayConnectionState::Connecting,
            GatewayConnectionState::Connected,
            GatewayConnectionState::Reconnecting,
            GatewayConnectionState::Stopped,
        ] {
            assert_eq!(GatewayConnectionState::from_u8(state as u8), state);
        }
    }

    #[test]
    fn disconnect_updates_shared_shutdown_signal() {
        let config = GatewayConfig::new("token").expect("valid config");
        let gateway = super::GatewayClient::new(config);

        assert!(!*gateway.shutdown_receiver.borrow());
        gateway.disconnect();
        assert!(*gateway.shutdown_receiver.borrow());
    }

    #[tokio::test]
    async fn bounded_event_queue_reports_overflow() {
        let mut config = GatewayConfig::new("token").expect("valid config");
        config.event_queue_capacity = 1;
        let gateway = super::GatewayClient::new(config);
        let mut events = gateway.events();

        gateway.enqueue_result(Ok(GatewayEvent::Authenticated));
        gateway.enqueue_result(Ok(GatewayEvent::Authenticated));

        let overflow = events.next().await.expect("overflow notification");
        assert!(matches!(
            overflow,
            Err(KahoError::GatewayEventQueueOverflow { dropped: 1 })
        ));
        assert!(matches!(
            events.next().await,
            Some(Ok(GatewayEvent::Authenticated))
        ));
    }

    #[cfg(feature = "cache")]
    #[tokio::test]
    async fn gateway_updates_cache_before_application_polling() {
        use std::sync::atomic::AtomicBool;

        use crate::{cache::Cache, models::Message};

        let config = GatewayConfig::new("token").expect("valid config");
        let mut gateway = super::GatewayClient::new(config);
        let cache = Cache::new();
        gateway.set_cache(cache.clone());
        let authenticated = AtomicBool::new(false);
        let message = Message {
            id: "message".to_owned(),
            nonuce: None,
            channel: "channel".to_owned(),
            author: "user".to_owned(),
            content: "hello".to_owned(),
            attachments: Vec::new(),
            embeds: None,
            mentions: Vec::new(),
            replies: Vec::new(),
        };

        gateway
            .process_gateway_event(GatewayEvent::Message(message), &authenticated)
            .await
            .expect("process event");

        assert!(cache.message("message").await.is_some());
    }
}
