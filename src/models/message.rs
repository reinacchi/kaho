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
    /// The ID of the message.
    #[serde(rename = "_id")]
    pub id: Id,
    pub nonuce: Option<String>,
    pub channel: Id,
    pub author: String,
    pub content: String,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub embeds: Option<Vec<Embed>>,
    #[serde(default)]
    pub mentions: Vec<Id>,
    #[serde(default)]
    pub replies: Vec<Id>,
}

impl Message {
    /// Edit this message.
    pub async fn edit(
        &self,
        http: &HttpClient,
        payload: impl Into<MessageEdit>,
    ) -> KahoResult<Self> {
        http.edit_message(&self.channel, &self.id, payload).await
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
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Id>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<EmbedCreate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub interactions: Vec<MessageInteractions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masquerade: Option<MessageMasquerade>,
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

// Message display masquerade information.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MessageMasquerade {
    pub avatar: Option<String>,
    pub colour: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MessageReplyIntent {
    pub fail_if_not_exists: bool,
    pub id: Id,
    pub mention: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct MessageInteractions {
    pub reactions: Vec<Id>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<EmbedCreate>,
}
