use serde::{Deserialize, Serialize};

use crate::{http::HttpClient, models::Id, KahoResult};

/// Custom emoji object.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Emoji {
    /// Emoji ID.
    #[serde(rename = "_id")]
    pub id: Id,
    /// Parent object that owns the emoji.
    pub parent: EmojiParent,
    /// User ID of the creator.
    pub creator_id: Id,
    /// Emoji name.
    pub name: String,
    /// Whether the emoji is animated.
    pub animated: bool,
    /// Whether the emoji is marked NSFW.
    pub nsfw: bool,
}

impl Emoji {
    /// Fetch a fresh copy of this emoji.
    pub async fn fetch(&self, http: &HttpClient) -> KahoResult<Self> {
        http.fetch_emoji(&self.id).await
    }

    /// Delete this emoji.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_emoji(&self.id).await
    }
}

/// Parent object for a custom emoji.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum EmojiParent {
    /// Server-owned emoji.
    Server { id: Id },
}

/// Payload for creating a custom emoji from an Autumn upload ID.
#[derive(Clone, Debug, Serialize)]
pub struct EmojiCreate {
    /// Emoji name.
    pub name: String,
    /// Emoji parent.
    pub parent: EmojiParent,
    /// Whether the emoji is mature.
    #[serde(default)]
    pub nsfw: bool,
}
