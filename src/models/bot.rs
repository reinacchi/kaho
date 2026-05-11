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
    /// The username displayed for the user or bot account.
    pub username: String,
    /// The bot avatar attachment, when present.
    pub avatar: Option<Attachment>,
    /// The human-readable description attached to the `PublicBot`.
    pub description: Option<String>,
}

impl PublicBot {
    /// Invite this public bot to a server.
    pub async fn invite(
        &self,
        http: &HttpClient,
        server_id: impl Into<Id>,
    ) -> KahoResult<BotInviteResponse> {
        http.invite_bot(
            &self.id,
            BotInvite {
                server: server_id.into(),
            },
        )
        .await
    }
}

/// Full bot object returned by authenticated bot management endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Bot {
    /// The unique ID assigned to the `Bot` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The ID of the user or account that owns the `Bot`.
    pub owner: Id,
    /// The token used to authenticate or execute this API resource.
    pub token: String,
    /// Whether this bot can be invited by other users.
    #[serde(rename = "public")]
    pub public_bot: bool,
    /// Whether analytics are enabled for this bot.
    #[serde(default)]
    pub analytics: bool,
    /// The discoverable value associated with this bot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<BotDiscoverable>,
}

impl Bot {
    /// Fetch a fresh copy of this bot.
    pub async fn fetch(&self, http: &HttpClient) -> KahoResult<Self> {
        http.fetch_bot(&self.id).await
    }

    /// Edit the `Bot`.
    pub async fn edit(&self, http: &HttpClient, payload: impl Into<BotUpdate>) -> KahoResult<Self> {
        http.edit_bot(&self.id, payload.into()).await
    }

    /// Delete the `Bot`.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_bot(&self.id).await
    }

    /// Invite this public bot to a server.
    pub async fn invite(
        &self,
        http: &HttpClient,
        server_id: impl Into<Id>,
    ) -> KahoResult<BotInviteResponse> {
        http.invite_bot(
            &self.id,
            BotInvite {
                server: server_id.into(),
            },
        )
        .await
    }
}

/// Represents a bot discoverable value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotDiscoverable {
    /// The human-readable description attached to the `BotDiscoverable`.
    pub description: String,
    /// The tags value associated with this bot discoverable.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Represents a bot create value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct BotCreate {
    /// The display name or configured name for the `BotCreate`.
    pub name: String,
}

impl From<String> for BotCreate {
    fn from(name: String) -> Self {
        Self { name }
    }
}

impl From<&str> for BotCreate {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

/// Response returned when creating a bot.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotCreateResponse {
    /// The ID of the user associated with the `BotCreateResponse`.
    pub user: User,
    /// The bot value associated with this bot create response.
    #[serde(flatten)]
    pub bot: Bot,
}

/// Represents a bot update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Serialize)]
pub struct BotUpdate {
    /// The display name or configured name for the `BotUpdate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The public bot value associated with this bot update.
    #[serde(rename = "public", skip_serializing_if = "Option::is_none")]
    pub public_bot: Option<bool>,
    /// The analytics value associated with this bot update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,
    /// The discoverable value associated with this bot update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<BotDiscoverable>,
    /// Fields to remove from the bot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<BotFields>,
}

/// Bot fields that can be removed by an update.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum BotFields {
    /// Represents the discoverable variant for this public enum.
    Discoverable,
}

/// Payload for inviting a bot to a server.
#[derive(Clone, Debug, Serialize)]
pub struct BotInvite {
    /// The ID of the server associated with the `BotInvite`.
    pub server: Id,
}

/// Response returned when a bot invite is accepted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotInviteResponse {
    /// The ID of the server associated with the `BotInviteResponse`.
    pub server: Id,
}
