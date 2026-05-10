use serde::{Deserialize, Serialize};

use crate::models::Id;

/// Represents a channel in a server.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Channel {
    DirectMessage(DirectMessageChannel),
}

/// Represents the fields that can be included in a channel object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChannelFields {
    DefaultPermissions,
    Description,
    Icon,
}

/// Represents a request to create a new channel in a server.
#[derive(Clone, Debug, Serialize)]
pub struct ChannelCreate {
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/// Represents the type of a channel.
#[derive(Clone, Debug, Serialize)]
pub enum ChannelType {
    Text,
    Voice,
}

/// Represents a request to update an existing channel in a server.
#[derive(Clone, Debug, Serialize)]
pub struct ChannelUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<ChannelFields>,
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
