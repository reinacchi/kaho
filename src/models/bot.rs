use serde::Deserialize;

use crate::models::Id;

/// Represents a public bot.
#[derive(Clone, Debug, Deserialize)]
pub struct PublicBot {
    /// The ID of the bot.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The username of the bot.
    pub username: String,
    /// The avatar of the bot.
    pub avatar: Option<String>,
    /// The description of the bot.
    pub description: Option<String>,

}
