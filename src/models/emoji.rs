use serde::{Deserialize, Serialize};

use crate::{http::HttpClient, models::Id, KahoResult};

/// Represents an emoji value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Emoji {
    /// The unique ID assigned to this resource by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// Parent object that owns the emoji.
    pub parent: EmojiParent,
    /// The creator ID value associated with this emoji.
    pub creator_id: Id,
    /// The display name or configured name for this resource.
    pub name: String,
    /// The animated value associated with this emoji.
    pub animated: bool,
    /// Whether the emoji is marked NSFW.
    pub nsfw: bool,
}

impl Emoji {
    /// Fetch a fresh copy of this emoji.
    pub async fn fetch(&self, http: &HttpClient) -> KahoResult<Self> {
        http.fetch_emoji(&self.id).await
    }

    /// Calls the Stoat API or client internals to delete for this resource.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_emoji(&self.id).await
    }
}

/// Represents the supported emoji parent variants returned by or sent to the Stoat API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum EmojiParent {
    /// Represents the server variant for this public enum.
    Server { id: Id },
}

/// Payload for creating a custom emoji from an Autumn upload ID.
#[derive(Clone, Debug, Serialize)]
pub struct EmojiCreate {
    /// The display name or configured name for this resource.
    pub name: String,
    /// The parent value associated with this emoji create.
    pub parent: EmojiParent,
    /// Whether this resource is marked as not safe for work.
    #[serde(default)]
    pub nsfw: bool,
}
