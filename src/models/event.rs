use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::AuthError,
    models::{Channel, Emoji, Id, Member, Message, RelationshipStatus, Server, User, UserFlags},
};

/// Events sent from the client to the Stoat gateway.
#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ClientEvent {
    /// Authenticate the WebSocket session with a bot token or user session token.
    Authenticate {
        /// Token used for authentication.
        token: String,
    },
    /// Notify the gateway that the client has started typing in a channel.
    BeginTyping {
        /// Channel ID where typing has started.
        channel: Id,
    },
    /// Notify the gateway that the client has stopped typing in a channel.
    EndTyping {
        /// Channel ID where typing has stopped.
        channel: Id,
    },
    /// Send a heartbeat ping to the gateway.
    Ping {
        /// Opaque heartbeat payload echoed by the gateway.
        data: usize,
    },
    /// Subscribe to a server's user update events.
    Subscribe {
        /// Server ID to subscribe to.
        server_id: Id,
    },
}

/// Data returned by the gateway Ready event.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ReadyEvent {
    /// Users sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub users: Vec<Value>,
    /// Servers sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub servers: Vec<Value>,
    /// Channels sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub channels: Vec<Value>,
    /// Server members sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub members: Vec<Value>,
    /// Custom emojis sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub emojis: Vec<Value>,
    /// User settings sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub user_settings: Vec<Value>,
    /// Channel unread state sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub channel_unreads: Vec<Value>,
    /// Policy changes sent in the initial gateway state.
    #[serde(default, deserialize_with = "deserialize_value_array")]
    pub policy_changes: Vec<Value>,
}

/// Identifier for a server member, as used by member update events.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemberId {
    /// Server ID containing the member.
    pub server: Id,
    /// User ID for the member.
    pub user: Id,
}

/// Message update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageUpdateEvent {
    /// Message ID that changed.
    pub id: Id,
    /// Channel ID containing the message.
    pub channel: Id,
    /// Partial Message object containing changed fields.
    pub data: Value,
}

/// Message append payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageAppendEvent {
    /// Message ID that received appended data.
    pub id: Id,
    /// Channel ID containing the message.
    pub channel: Id,
    /// Appended message data, such as new embeds.
    pub append: Value,
}

/// Message deletion payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageDeleteEvent {
    /// Message ID that was deleted.
    pub id: Id,
    /// Channel ID containing the message.
    pub channel: Id,
}

/// Message reaction payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageReactionEvent {
    /// Message ID whose reaction set changed.
    pub id: Id,
    /// Channel ID containing the message.
    pub channel_id: Id,
    /// User ID that added or removed the reaction.
    pub user_id: Id,
    /// Emoji ID for the reaction.
    pub emoji_id: Id,
}

/// Message reaction removal payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageRemoveReactionEvent {
    /// Message ID whose reaction was removed.
    pub id: Id,
    /// Channel ID containing the message.
    pub channel_id: Id,
    /// Emoji ID for the removed reaction.
    pub emoji_id: Id,
}

/// Channel update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelUpdateEvent {
    /// Channel ID that changed.
    pub id: Id,
    /// Partial Channel object containing changed fields.
    pub data: Value,
    /// Fields cleared from the channel.
    #[serde(default)]
    pub clear: Vec<String>,
}

/// Channel deletion payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelDeleteEvent {
    /// Channel ID that was deleted.
    pub id: Id,
}

/// Group channel membership event payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelGroupMemberEvent {
    /// Channel ID whose membership changed.
    pub id: Id,
    /// User ID that joined or left.
    pub user: Id,
}

/// Channel typing event payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelTypingEvent {
    /// Channel ID where typing changed.
    pub id: Id,
    /// User ID whose typing state changed.
    pub user: Id,
}

/// Channel acknowledgement payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChannelAckEvent {
    /// Channel ID acknowledged by the user.
    pub id: Id,
    /// User ID that acknowledged messages.
    pub user: Id,
    /// Last acknowledged message ID.
    pub message_id: Id,
}

/// Server update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerUpdateEvent {
    /// Server ID that changed.
    pub id: Id,
    /// Partial Server object containing changed fields.
    pub data: Value,
    /// Fields cleared from the server.
    #[serde(default)]
    pub clear: Vec<String>,
}

/// Server deletion payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerDeleteEvent {
    /// Server ID that was deleted.
    pub id: Id,
}

/// Server member update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerMemberUpdateEvent {
    /// Compound member ID containing server and user IDs.
    pub id: MemberId,
    /// Partial Member object containing changed fields.
    pub data: Value,
    /// Fields cleared from the member.
    #[serde(default)]
    pub clear: Vec<String>,
}

/// Server member join payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerMemberJoinEvent {
    /// Server ID that the user joined.
    pub id: Id,
    /// User ID that joined.
    pub user: Id,
    /// Full member object for the joining user.
    pub member: Member,
}

/// Server member leave payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerMemberLeaveEvent {
    /// Server ID that the user left.
    pub id: Id,
    /// User ID that left.
    pub user: Id,
}

/// Server role update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerRoleUpdateEvent {
    /// Server ID containing the role.
    pub id: Id,
    /// Role ID that changed.
    pub role_id: Id,
    /// Partial Role object containing changed fields.
    pub data: Value,
    /// Fields cleared from the role.
    #[serde(default)]
    pub clear: Vec<String>,
}

/// Server role deletion payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerRoleDeleteEvent {
    /// Server ID containing the role.
    pub id: Id,
    /// Role ID that was deleted.
    pub role_id: Id,
}

/// User update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserUpdateEvent {
    /// User ID that changed.
    pub id: Id,
    /// Partial User object containing changed fields.
    pub data: Value,
    /// Fields cleared from the user.
    #[serde(default)]
    pub clear: Vec<String>,
}

/// Relationship update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserRelationshipEvent {
    /// Current user ID.
    pub id: Id,
    /// User object whose relationship changed.
    pub user: User,
    /// New relationship status.
    pub status: RelationshipStatus,
}

/// User platform wipe payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserPlatformWipeEvent {
    /// User ID whose account data should be wiped locally.
    pub user_id: Id,
    /// User flags explaining why the wipe is occurring.
    pub flags: UserFlags,
}

/// Emoji update payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EmojiUpdateEvent {
    /// Emoji ID that changed.
    pub id: Id,
    /// Partial Emoji object containing changed fields.
    pub data: Value,
}

/// Emoji deletion payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EmojiDeleteEvent {
    /// Emoji ID that was deleted.
    pub id: Id,
}

/// Authifier event forwarded through the Stoat gateway.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "event_type")]
pub enum AuthEvent {
    /// A single session was deleted.
    DeleteSession {
        /// User ID whose session was deleted.
        user_id: Id,
        /// Deleted session ID.
        session_id: Id,
    },
    /// All sessions for a user were deleted.
    DeleteAllSessions {
        /// User ID whose sessions were deleted.
        user_id: Id,
        /// Optional session ID excluded from deletion.
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude_session_id: Option<Id>,
    },
}

/// Events received from the Stoat gateway.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum GatewayEvent {
    /// The gateway returned an authentication or protocol error.
    Error {
        /// Error reported by the gateway.
        error: AuthError,
    },
    /// The server has authenticated the connection.
    Authenticated,
    /// The current user session has been invalidated or the bot token was reset.
    #[serde(rename = "Logout")]
    LoggedOut,
    /// Several events have been sent together.
    Bulk {
        /// Events included in the bulk payload.
        #[serde(default)]
        v: Vec<GatewayEvent>,
    },
    /// Heartbeat pong received from the gateway.
    Pong {
        /// Opaque heartbeat payload echoed by the gateway.
        data: usize,
    },
    /// Initial state data sent after authentication.
    Ready(ReadyEvent),
    /// A message was received.
    Message(Message),
    /// A message was edited or otherwise updated.
    MessageUpdate(MessageUpdateEvent),
    /// Data was appended to a message.
    MessageAppend(MessageAppendEvent),
    /// A message was deleted.
    MessageDelete(MessageDeleteEvent),
    /// A reaction was added to a message.
    MessageReact(MessageReactionEvent),
    /// A reaction was removed from a message.
    MessageUnreact(MessageReactionEvent),
    /// A reaction was cleared from a message.
    MessageRemoveReaction(MessageRemoveReactionEvent),
    /// A channel was created.
    ChannelCreate(Channel),
    /// Channel details were updated.
    ChannelUpdate(ChannelUpdateEvent),
    /// A channel was deleted.
    ChannelDelete(ChannelDeleteEvent),
    /// A user joined a group channel.
    ChannelGroupJoin(ChannelGroupMemberEvent),
    /// A user left a group channel.
    ChannelGroupLeave(ChannelGroupMemberEvent),
    /// A user started typing in a channel.
    ChannelStartTyping(ChannelTypingEvent),
    /// A user stopped typing in a channel.
    ChannelStopTyping(ChannelTypingEvent),
    /// A user acknowledged messages in a channel.
    ChannelAck(ChannelAckEvent),
    /// A server was created.
    ServerCreate(Server),
    /// Server details were updated.
    ServerUpdate(ServerUpdateEvent),
    /// A server was deleted.
    ServerDelete(ServerDeleteEvent),
    /// A server member was updated.
    ServerMemberUpdate(ServerMemberUpdateEvent),
    /// A user joined a server.
    ServerMemberJoin(ServerMemberJoinEvent),
    /// A user left a server.
    ServerMemberLeave(ServerMemberLeaveEvent),
    /// A server role was updated or created.
    ServerRoleUpdate(ServerRoleUpdateEvent),
    /// A server role was deleted.
    ServerRoleDelete(ServerRoleDeleteEvent),
    /// User details were updated.
    UserUpdate(UserUpdateEvent),
    /// The current user's relationship with another user changed.
    UserRelationship(UserRelationshipEvent),
    /// A user has been platform banned or deleted their account.
    UserPlatformWipe(UserPlatformWipeEvent),
    /// An emoji was created.
    EmojiCreate(Emoji),
    /// Emoji details were updated.
    EmojiUpdate(EmojiUpdateEvent),
    /// An emoji was deleted.
    EmojiDelete(EmojiDeleteEvent),
    /// Authifier event forwarded through the gateway.
    Auth(AuthEvent),
    /// Any gateway event not yet modelled by Kaho.
    #[serde(other)]
    Unknown,
}

fn deserialize_value_array<'de, D>(deserializer: D) -> Result<Vec<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;

    Ok(match value {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(values)) => values,
        Some(value) => vec![value],
    })
}
