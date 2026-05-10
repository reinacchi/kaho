use crate::models::attachment::Attachment;
use serde::{Deserialize, Serialize};

/// Represents an embed in a message, which can be a website, image, video, text, or none.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Embed {
    Website(WebsiteMetadata),
    Image(Image),
    Video(Video),
    Text(Text),
    None,
}

/// Represents a request to create an embed in a message.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EmbedCreate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
}

/// The image embed.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Image {
    /// URL to the original image
    pub url: String,

    /// Width of the image
    pub width: isize,

    /// Height of the image
    pub height: isize,

    /// Positioning and size
    pub size: ImageSize,
}

/// Size of the image in the embed
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum ImageSize {
    /// Show large preview at the bottom of the embed
    Large,

    /// Show small preview to the side of the embed
    Preview,
}

/// The video embed.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Video {
    /// URL to the original video
    pub url: String,

    /// Width of the video
    pub width: isize,

    /// Height of the video
    pub height: isize,
}

/// The text embed.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Text {
    /// URL to icon
    pub icon_url: Option<String>,

    /// URL for title
    pub url: Option<String>,

    /// Title of text embed
    pub title: Option<String>,

    /// Description of text embed
    pub description: Option<String>,

    /// ID of uploaded attachment
    pub media: Option<Attachment>,

    /// CSS colour
    pub colour: Option<String>,
}

/// Represents special remote content that can be embedded in a message.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Special {
    /// No remote content
    None,

    /// Content hint that this contains a GIF
    ///
    /// Use metadata to find video or image to play
    GIF,

    /// YouTube video
    YouTube {
        id: String,
        timestamp: Option<String>,
    },

    /// Lightspeed.tv stream
    Lightspeed {
        content_type: LightspeedType,
        id: String,
    },

    /// Twitch stream or clip
    Twitch {
        content_type: TwitchType,
        id: String,
    },
    /// Spotify track
    Spotify { content_type: String, id: String },

    /// Soundcloud track
    Soundcloud,

    /// Bandcamp track
    Bandcamp {
        content_type: BandcampType,
        id: String,
    },
}

/// Type of remote Twitch content
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum TwitchType {
    Channel,
    Video,
    Clip,
}

/// Type of remote Lightspeed.tv content
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum LightspeedType {
    Channel,
}

/// Type of remote Bandcamp content
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum BandcampType {
    Album,
    Track,
}

/// Metadata for a website that can be embedded in a message.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct WebsiteMetadata {
    /// Direct URL to web page
    pub url: Option<String>,

    /// Original direct URL
    pub original_url: Option<String>,

    /// Remote content
    pub special: Option<Special>,

    /// Title of website
    pub title: Option<String>,

    /// Description of website
    pub description: Option<String>,

    /// Embedded image
    pub image: Option<Image>,

    /// Embedded video
    pub video: Option<Video>,

    /// Site name
    pub site_name: Option<String>,

    /// URL to site icon
    pub icon_url: Option<String>,

    /// CSS colour
    pub colour: Option<String>,
}
