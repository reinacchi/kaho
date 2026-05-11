use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    http::HttpClient,
    models::{
        Attachment, FetchMessageQuery, Id, Invite, Message, MessageEdit, MessageSearch,
        MessageSend, OverrideField, User,
    },
    KahoResult,
};

/// Represents the supported channel variants returned by or sent to the Stoat API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "channel_type")]
pub enum Channel {
    /// Saved messages channel for the current user.
    SavedMessages(SavedMessagesChannel),
    /// Represents the direct message variant for this public enum.
    DirectMessage(DirectMessageChannel),
    /// Represents the group variant for this public enum.
    Group(GroupChannel),
    /// Represents the text channel variant for this public enum.
    TextChannel(TextChannel),
    /// Represents the voice channel variant for this public enum.
    VoiceChannel(VoiceChannel),
}

/// Represents the fields that can be removed from a channel object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChannelFields {
    /// Represents the default permissions variant for this public enum.
    DefaultPermissions,
    /// Represents the description variant for this public enum.
    Description,
    /// Represents the icon variant for this public enum.
    Icon,
}

/// Represents a request to create a new channel in a server.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ChannelCreate {
    /// The display name or configured name for the `ChannelCreate`.
    pub name: String,
    /// The channel type value associated with this channel create.
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    /// The human-readable description attached to the `ChannelCreate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the `ChannelCreate` is marked as not safe for work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/// Represents a request to create a group channel.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GroupCreate {
    /// The display name or configured name for the `GroupCreate`.
    pub name: String,
    /// The user IDs included in this response or request payload.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<Id>,
    /// Whether the `GroupCreate` is marked as not safe for work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/// Represents the type of a channel to create in a server.
#[derive(Clone, Debug, Serialize)]
pub enum ChannelType {
    /// Represents the text variant for this public enum.
    Text,
    /// Represents the voice variant for this public enum.
    Voice,
}

impl Default for ChannelType {
    fn default() -> Self {
        Self::Text
    }
}

/// Represents a request to update an existing channel.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ChannelUpdate {
    /// Whether this channel is archived and hidden from normal active use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    /// The human-readable description attached to the `ChannelUpdate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The icon attachment or icon reference associated with the `ChannelUpdate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Id>,
    /// The display name or configured name for the `ChannelUpdate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the `ChannelUpdate` is marked as not safe for work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    /// The ID of the user or account that owns the `ChannelUpdate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Id>,
    /// The list of fields that should be removed from the resource during update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<ChannelFields>,
}

/// Represents a channel close query value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ChannelCloseQuery {
    /// The leave silently value associated with this channel close query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leave_silently: Option<bool>,
}

/// Represents a saved messages channel value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SavedMessagesChannel {
    /// The unique ID assigned to the `SavedMessagesChannel` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The ID of the user associated with the `SavedMessagesChannel`.
    pub user: Id,
    /// The ID of the most recent message known for this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<Id>,
}

/// Represents a direct message channel between two users.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DirectMessageChannel {
    /// Whether this channel or resource is currently active.
    pub active: bool,
    /// The unique ID assigned to the `DirectMessageChannel` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The ID of the most recent message known for this channel.
    pub last_message_id: Option<Id>,
    /// The user IDs that participate in this group or direct message channel.
    pub recipients: [Id; 2],
}

/// Represents a group channel value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GroupChannel {
    /// The unique ID assigned to the `GroupChannel` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The display name or configured name for the `GroupChannel`.
    pub name: String,
    /// The ID of the user or account that owns the `GroupChannel`.
    pub owner: Id,
    /// The user IDs that participate in this group or direct message channel.
    #[serde(default)]
    pub recipients: Vec<Id>,
    /// The human-readable description attached to the `GroupChannel`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The icon attachment or icon reference associated with the `GroupChannel`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    /// The ID of the most recent message known for this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<Id>,
    /// Whether the `GroupChannel` is marked as not safe for work.
    #[serde(default)]
    pub nsfw: bool,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a text channel value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TextChannel {
    /// The unique ID assigned to the `TextChannel` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The ID of the server associated with the `TextChannel`.
    pub server: Id,
    /// The display name or configured name for the `TextChannel`.
    pub name: String,
    /// The human-readable description attached to the `TextChannel`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The icon attachment or icon reference associated with the `TextChannel`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    /// The ID of the most recent message known for this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<Id>,
    /// Whether the `TextChannel` is marked as not safe for work.
    #[serde(default)]
    pub nsfw: bool,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a voice channel value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VoiceChannel {
    /// The unique ID assigned to the `VoiceChannel` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The ID of the server associated with the `VoiceChannel`.
    pub server: Id,
    /// The display name or configured name for the `VoiceChannel`.
    pub name: String,
    /// The human-readable description attached to the `VoiceChannel`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The icon attachment or icon reference associated with the `VoiceChannel`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    /// Whether the `VoiceChannel` is marked as not safe for work.
    #[serde(default)]
    pub nsfw: bool,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Response returned after joining a voice call.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct JoinCallResponse {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

impl Channel {
    /// Return the `Channel` ID.
    pub fn id(&self) -> &str {
        match self {
            Channel::SavedMessages(channel) => &channel.id,
            Channel::DirectMessage(channel) => &channel.id,
            Channel::Group(channel) => &channel.id,
            Channel::TextChannel(channel) => &channel.id,
            Channel::VoiceChannel(channel) => &channel.id,
        }
    }

    /// Fetch a fresh copy of this channel.
    pub async fn fetch(&self, http: &HttpClient) -> KahoResult<Self> {
        http.fetch_channel(self.id()).await
    }

    /// Edit the `Channel`.
    pub async fn edit(
        &self,
        http: &HttpClient,
        payload: impl Into<ChannelUpdate>,
    ) -> KahoResult<Self> {
        http.edit_channel(self.id(), payload).await
    }

    /// Close, leave, or delete this channel depending on its type.
    pub async fn close(
        &self,
        http: &HttpClient,
        query: impl Into<Option<ChannelCloseQuery>>,
    ) -> KahoResult {
        http.close_channel(self.id(), query).await
    }

    /// Create an invite for this channel.
    pub async fn create_invite(&self, http: &HttpClient) -> KahoResult<Invite> {
        http.create_channel_invite(self.id()).await
    }

    /// Set permissions for a role in this channel.
    pub async fn set_role_permissions(
        &self,
        http: &HttpClient,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        http.set_channel_permissions(self.id(), role_id, payload)
            .await
    }

    /// Set default permissions for this channel.
    pub async fn set_default_permissions(
        &self,
        http: &HttpClient,
        payload: OverrideField,
    ) -> KahoResult {
        http.set_channel_default_permissions(self.id(), payload)
            .await
    }

    /// Acknowledge a message in this channel.
    pub async fn acknowledge_message(&self, http: &HttpClient, message_id: &str) -> KahoResult {
        http.acknowledge_message(self.id(), message_id).await
    }

    /// Fetch messages for the `Channel`.
    pub async fn messages(
        &self,
        http: &HttpClient,
        query: impl Into<Option<FetchMessageQuery>>,
    ) -> KahoResult<Vec<Message>> {
        http.fetch_messages(self.id(), query).await
    }

    /// Send a message to this channel.
    pub async fn send_message(
        &self,
        http: &HttpClient,
        payload: impl Into<MessageSend>,
    ) -> KahoResult<Message> {
        http.send_message(self.id(), payload).await
    }

    /// Search messages in the `Channel`.
    pub async fn search_messages(
        &self,
        http: &HttpClient,
        payload: impl Into<MessageSearch>,
    ) -> KahoResult<Vec<Message>> {
        http.search_messages(self.id(), payload).await
    }

    /// Fetch one message from this channel.
    pub async fn message(&self, http: &HttpClient, message_id: &str) -> KahoResult<Message> {
        http.fetch_message(self.id(), message_id).await
    }

    /// Edit one message in this channel.
    pub async fn edit_message(
        &self,
        http: &HttpClient,
        message_id: &str,
        payload: impl Into<MessageEdit>,
    ) -> KahoResult<Message> {
        http.edit_message(self.id(), message_id, payload).await
    }

    /// Delete one message from this channel.
    pub async fn delete_message(&self, http: &HttpClient, message_id: &str) -> KahoResult {
        http.delete_message(self.id(), message_id).await
    }

    /// Fetch group members for this channel.
    pub async fn members(&self, http: &HttpClient) -> KahoResult<Vec<User>> {
        http.fetch_group_members(self.id()).await
    }

    /// Add a recipient to this group channel.
    pub async fn add_recipient(&self, http: &HttpClient, user_id: &str) -> KahoResult {
        http.add_group_recipient(self.id(), user_id).await
    }

    /// Remove a recipient from this group channel.
    pub async fn remove_recipient(&self, http: &HttpClient, user_id: &str) -> KahoResult {
        http.remove_group_recipient(self.id(), user_id).await
    }

    /// Join this voice channel or call.
    pub async fn join_call(&self, http: &HttpClient) -> KahoResult<JoinCallResponse> {
        http.join_call(self.id()).await
    }

    /// Stop ringing a user for this channel or call.
    pub async fn end_ring(&self, http: &HttpClient, user_id: &str) -> KahoResult {
        http.end_ring(self.id(), user_id).await
    }
}
