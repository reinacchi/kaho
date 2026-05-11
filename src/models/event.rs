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
    /// Represents the authenticated variant for this public enum.
    Authenticated,
    /// The gateway returned an authentication or protocol error.
    Error {
        /// Error reported by the gateway.
        error: AuthError,
    },
    /// Heartbeat pong received from the gateway.
    Pong,
    /// Represents the ready variant for this public enum.
    Ready,
    /// Represents the message variant for this public enum.
    Message(Message),
    /// Represents the server create variant for this public enum.
    ServerCreate(Server),
    /// Represents the channel start typing variant for this public enum.
    ChannelStartTyping,
    /// Represents the channel stop typing variant for this public enum.
    ChannelStopTyping,
    /// Any gateway event not yet modelled by Kaho.
    #[serde(other)]
    Unknown,
}
