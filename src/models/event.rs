use serde::{Deserialize, Serialize};

use crate::{
    error::AuthError,
    models::{Message, Server, Id},
};

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ClientEvent {
    Authenticate { token: String },
    BeginTyping { channel: Id },
    EndTyping { channel: Id },
    Ping { data: usize },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum GatewayEvent {
    Authenticated,
    Error { error: AuthError },
    Pong,
    Ready,
    Message(Message),
    ServerCreate(Server),
    ChannelStartTyping,
    ChannelStopTyping,
    #[serde(other)]
    Unknown,
}
