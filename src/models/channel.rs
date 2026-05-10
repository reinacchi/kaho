use serde::{Deserialize, Serialize};

use crate::models::Id;

/// Represents a channel in a server.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Channel {
    /// A direct message channel between two users.
    DirectMessage(DirectMessageChannel),
}

/// Represents the fields that can be included in a channel object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChannelFields {
    /// Default permission settings.
    DefaultPermissions,
    /// Channel description.
    Description,
    /// Channel icon.
    Icon,
}

/// Represents a request to create a new channel in a server.
#[derive(Clone, Debug, Serialize)]
pub struct ChannelCreate {
    /// Name of the channel to create.
    pub name: String,
    /// Type of channel to create.
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    /// Optional channel description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the channel should be marked as NSFW.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/// Represents the type of a channel.
#[derive(Clone, Debug, Serialize)]
pub enum ChannelType {
    /// Text channel.
    Text,
    /// Voice channel.
    Voice,
}

/// Represents a request to update an existing channel in a server.
#[derive(Clone, Debug, Serialize)]
pub struct ChannelUpdate {
    /// Whether the channel should be archived.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    /// New channel description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New channel icon ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Id>,
    /// New channel name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the channel is NSFW.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    /// New channel owner ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Id>,
    /// Field to remove from the channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<ChannelFields>,
}

impl Channel {
    /// Return the channel ID, independent of the channel variant.
    pub fn id(&self) -> &str {
        match self {
            Channel::DirectMessage(channel) => &channel.id,
        }
    }
}

/// Represents a direct message channel between two users.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DirectMessageChannel {
    /// Whether the direct message is active.
    pub active: bool,
    /// The ID of the channel.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The ID of the last message in the direct message channel.
    pub last_message_id: Option<Id>,
    /// The recipients of the direct message.
    pub recipients: [Id; 2],
}
