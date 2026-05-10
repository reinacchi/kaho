use serde::{Deserialize, Serialize};

use crate::{
    error::AuthError,
    models::{Id, Message, Server},
};

/// Events sent from the client to the Stoat gateway.
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ClientEvent {
    /// Authenticate the WebSocket session with a bot token.
    Authenticate {
        /// Bot token used for authentication.
        token: String,
    },
    /// Notify the gateway that the bot has started typing in a channel.
    BeginTyping {
        /// Channel ID where typing has started.
        channel: Id,
    },
    /// Notify the gateway that the bot has stopped typing in a channel.
    EndTyping {
        /// Channel ID where typing has stopped.
        channel: Id,
    },
    /// Send a heartbeat ping to the gateway.
    Ping {
        /// Opaque heartbeat payload.
        data: usize,
    },
}

/// Events received from the Stoat gateway.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum GatewayEvent {
    /// The gateway accepted authentication.
    Authenticated,
    /// The gateway returned an authentication or protocol error.
    Error {
        /// Error reported by the gateway.
        error: AuthError,
    },
    /// Heartbeat pong received from the gateway.
    Pong,
    /// Initial ready event after connecting.
    Ready,
    /// A message was created.
    Message(Message),
    /// A server was created or became available.
    ServerCreate(Server),
    /// A user started typing in a channel.
    ChannelStartTyping,
    /// A user stopped typing in a channel.
    ChannelStopTyping,
    /// Any gateway event not yet modelled by Kaho.
    #[serde(other)]
    Unknown,
}
