use serde::{Deserialize, Serialize};

use crate::{
    http::HttpClient,
    models::{Attachment, Id, User},
    KahoResult,
};

/// Public bot information returned by the bot invite endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PublicBot {
    /// The ID of the bot user.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The bot username.
    pub username: String,
    /// The bot avatar attachment, when present.
    pub avatar: Option<Attachment>,
    /// Public bot description.
    pub description: Option<String>,
}

impl PublicBot {
    /// Invite this public bot to a server.
    pub async fn invite(&self, http: &HttpClient, server_id: impl Into<Id>) -> KahoResult<BotInviteResponse> {
        http.invite_bot(&self.id, BotInvite { server: server_id.into() }).await
    }
}

/// Full bot object returned by authenticated bot management endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Bot {
    /// The bot ID.
    #[serde(rename = "_id")]
    pub id: Id,
    /// Owner account ID.
    pub owner: Id,
    /// Bot token.
    pub token: String,
    /// Whether this bot can be invited by other users.
    #[serde(rename = "public")]
    pub public_bot: bool,
    /// Whether analytics are enabled for this bot.
    #[serde(default)]
    pub analytics: bool,
    /// Discoverability metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<BotDiscoverable>,
}

impl Bot {
    /// Fetch a fresh copy of this bot.
    pub async fn fetch(&self, http: &HttpClient) -> KahoResult<Self> {
        http.fetch_bot(&self.id).await
    }

    /// Edit this bot.
    pub async fn edit(&self, http: &HttpClient, payload: impl Into<BotUpdate>) -> KahoResult<Self> {
        http.edit_bot(&self.id, payload.into()).await
    }

    /// Delete this bot.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_bot(&self.id).await
    }

    /// Invite this public bot to a server.
    pub async fn invite(&self, http: &HttpClient, server_id: impl Into<Id>) -> KahoResult<BotInviteResponse> {
        http.invite_bot(&self.id, BotInvite { server: server_id.into() }).await
    }
}

/// Discoverability metadata attached to a bot.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotDiscoverable {
    /// Bot description.
    pub description: String,
    /// Search tags for the bot.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Payload for creating a bot.
#[derive(Clone, Debug, Serialize)]
pub struct BotCreate {
    /// Bot username.
    pub name: String,
}

impl From<String> for BotCreate {
    fn from(name: String) -> Self {
        Self { name }
    }
}

impl From<&str> for BotCreate {
    fn from(name: &str) -> Self {
        Self { name: name.to_owned() }
    }
}

/// Response returned when creating a bot.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotCreateResponse {
    /// Created bot user.
    pub user: User,
    /// Created bot metadata.
    #[serde(flatten)]
    pub bot: Bot,
}

/// Payload for editing a bot.
#[derive(Clone, Debug, Default, Serialize)]
pub struct BotUpdate {
    /// Replacement bot name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the bot is public.
    #[serde(rename = "public", skip_serializing_if = "Option::is_none")]
    pub public_bot: Option<bool>,
    /// Whether analytics are enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,
    /// Discoverability settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<BotDiscoverable>,
    /// Fields to remove from the bot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<BotFields>,
}

/// Bot fields that can be removed by an update.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum BotFields {
    /// Remove discoverability metadata.
    Discoverable,
}

/// Payload for inviting a bot to a server.
#[derive(Clone, Debug, Serialize)]
pub struct BotInvite {
    /// Target server ID.
    pub server: Id,
}

/// Response returned when a bot invite is accepted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotInviteResponse {
    /// Server ID the bot joined.
    pub server: Id,
}
