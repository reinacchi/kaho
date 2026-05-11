use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::{
    http::HttpClient,
    models::{
        attachment::Attachment,
        embed::{Embed, EmbedCreate},
        Id,
    },
    KahoResult,
};

/// Represents a message in the Stoat platform.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    /// The unique ID assigned to the `Message` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// Optional nonce supplied when the message was created.
    pub nonuce: Option<String>,
    /// ID of the channel containing the message.
    pub channel: Id,
    /// ID of the user that authored the message.
    pub author: String,
    /// The textual content included in this message or payload.
    pub content: String,
    /// The attachments included with this message or webhook payload.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// The embeds value associated with this message.
    #[serde(default)]
    pub embeds: Option<Vec<Embed>>,
    /// User IDs mentioned by the message.
    #[serde(default)]
    pub mentions: Vec<Id>,
    /// Message IDs replied to by this message.
    #[serde(default)]
    pub replies: Vec<Id>,
}

impl Message {
    /// Acknowledge the message corresponding to this instance.
    pub async fn acknowledge(&self, http: &HttpClient) -> KahoResult {
        http.acknowledge_message(&self.channel, &self.id).await
    }

    /// Delete the message corresponding to this instance.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_message(&self.channel, &self.id).await
    }

    /// Pin the message corresponding to this instance.
    pub async fn pin(&self, http: &HttpClient) -> KahoResult {
        http.pin_message(&self.channel, &self.id).await
    }

    /// Unpin the message corresponding to this instance.
    pub async fn unpin(&self, http: &HttpClient) -> KahoResult {
        http.unpin_message(&self.channel, &self.id).await
    }

    /// Edit the `Message`.
    pub async fn edit(
        &self,
        http: &HttpClient,
        payload: impl Into<MessageEdit>,
    ) -> KahoResult<Self> {
        http.edit_message(&self.channel, &self.id, payload).await
    }

    /// Add a reaction to this message.
    pub async fn react(&self, http: &HttpClient, emoji: &str) -> KahoResult {
        http.add_reaction(&self.channel, &self.id, emoji).await
    }

    /// Remove this client's reaction from this message.
    pub async fn remove_reaction(&self, http: &HttpClient, emoji: &str) -> KahoResult {
        http.remove_reaction(&self.channel, &self.id, emoji).await
    }

    /// Clear message reactions.
    pub async fn clear_reactions(&self, http: &HttpClient) -> KahoResult {
        http.clear_reactions(&self.channel, &self.id).await
    }

    /// Reply to the message corresponding to this instance.
    pub async fn reply(
        &self,
        http: &HttpClient,
        payload: impl Into<MessageSend>,
        mention: bool,
    ) -> KahoResult<Self> {
        http.reply_message(&self.channel, &self.id, payload, mention)
            .await
    }
}

/// Represents a request to create a new message.
#[derive(Clone, Debug, Default, Serialize)]
pub struct MessageSend {
    /// Text content of the outgoing message.
    pub content: String,
    /// The attachments included with this message or webhook payload.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Id>,
    /// The embeds value associated with this message send.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<EmbedCreate>,
    /// The flags value associated with this message send.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<MessageFlags>,
    /// The interactions value associated with this message send.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub interactions: Vec<MessageInteractions>,
    /// The masquerade value associated with this message send.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masquerade: Option<MessageMasquerade>,
    /// The replies value associated with this message send.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<MessageReplyIntent>,
}

bitflags! {
    /// Represents the flags associated with a message.
    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
    pub struct MessageFlags: u32 {
        const SurpressNotifications = 1;
        const MentionsEveryone = 2;
        const MentionsOnline = 3;
    }
}

/// Represents a message masquerade value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageMasquerade {
    /// Avatar URL to display for the masquerade.
    pub avatar: Option<String>,
    /// The role or embed colour value encoded as an API integer.
    pub colour: Option<String>,
    /// The display name or configured name for the `MessageMasquerade`.
    pub name: String,
}
/// Represents the supported search message sort variants returned by or sent to the Stoat API.

#[derive(Clone, Debug, Serialize)]
pub enum SearchMessageSort {
    /// Represents the relevance variant for this public enum.
    Relevance,
    /// Represents the newest variant for this public enum.
    Newest,
    /// Represents the oldest variant for this public enum.
    Oldest,
}

impl std::fmt::Display for SearchMessageSort {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SearchMessageSort::Relevance => write!(f, "Relevance"),
            SearchMessageSort::Newest => write!(f, "Newest"),
            SearchMessageSort::Oldest => write!(f, "Oldest"),
        }
    }
}
/// Represents a fetch message query value used by the Stoat API models and endpoints.

#[derive(Clone, Debug, Serialize)]
pub struct FetchMessageQuery {
    /// ID of the message to search after.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,

    /// ID of the message to search before.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    /// Maximum number of messages to return. Must be between 1 and 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// The sort order for the search results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<SearchMessageSort>,
}
/// Represents a message search value used by the Stoat API models and endpoints.

#[derive(Clone, Debug, Serialize)]
pub struct MessageSearch {
    /// ID of the message to search after.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,

    /// ID of the message to search before.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    /// Whether to include the content of the message in the search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_users: Option<bool>,

    /// Maximum number of messages to return. Must be between 1 and 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// Whether to only search for pinned messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,

    /// The query value associated with this message search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// The sort order for the search results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<SearchMessageSort>,
}

/// Represents a message reply intent value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct MessageReplyIntent {
    /// Whether sending should fail when the replied-to message does not exist.
    pub fail_if_not_exists: bool,
    /// ID of the message being replied to.
    pub id: Id,
    /// Whether the reply should mention the original author.
    pub mention: bool,
}

/// Represents a message interactions value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct MessageInteractions {
    /// Reaction IDs allowed on the message.
    pub reactions: Vec<Id>,
    /// Whether reactions are restricted to the listed IDs.
    pub restrict_reactions: bool,
}

impl<T: Into<String>> From<T> for MessageSend {
    fn from(content: T) -> Self {
        Self {
            content: content.into(),
            attachments: Vec::new(),
            embeds: Vec::new(),
            flags: None,
            interactions: Vec::new(),
            masquerade: None,
            replies: Vec::new(),
        }
    }
}

/// Represents a request to edit an existing message.
#[derive(Clone, Debug, Default, Serialize)]
pub struct MessageEdit {
    /// The textual content included in this message or payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The embeds value associated with this message edit.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<EmbedCreate>,
}
