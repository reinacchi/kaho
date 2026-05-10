use crate::models::attachment::Attachment;
use serde::{Deserialize, Serialize};

/// Represents an embed in a message, which can be a website, image, video, text, or none.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Embed {
    /// Website metadata embed.
    Website(WebsiteMetadata),
    /// Image embed.
    Image(Image),
    /// Video embed.
    Video(Video),
    /// Text embed.
    Text(Text),
    /// Empty embed placeholder.
    None,
}

/// Represents a request to create an embed in a message.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EmbedCreate {
    /// Icon URL displayed with the embed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// URL attached to the embed title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Embed title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Embed description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Attachment ID or media reference for the embed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<String>,
    /// Embed accent colour.
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
        /// YouTube video ID.
        id: String,
        /// Optional playback timestamp.
        timestamp: Option<String>,
    },

    /// Lightspeed.tv stream
    Lightspeed {
        /// Lightspeed content type.
        content_type: LightspeedType,
        /// Lightspeed content ID.
        id: String,
    },

    /// Twitch stream or clip
    Twitch {
        /// Twitch content type.
        content_type: TwitchType,
        /// Twitch content ID.
        id: String,
    },
    /// Spotify track
    Spotify {
        /// Spotify content type.
        content_type: String,
        /// Spotify content ID.
        id: String,
    },

    /// Soundcloud track
    Soundcloud,

    /// Bandcamp track
    Bandcamp {
        /// Bandcamp content type.
        content_type: BandcampType,
        /// Bandcamp content ID.
        id: String,
    },
}

/// Type of remote Twitch content
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum TwitchType {
    /// Twitch channel.
    Channel,
    /// Twitch video.
    Video,
    /// Twitch clip.
    Clip,
}

/// Type of remote Lightspeed.tv content
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum LightspeedType {
    /// Lightspeed channel.
    Channel,
}

/// Type of remote Bandcamp content
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum BandcampType {
    /// Bandcamp album.
    Album,
    /// Bandcamp track.
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
