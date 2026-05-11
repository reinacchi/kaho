use crate::models::attachment::Attachment;
use serde::{Deserialize, Serialize};

/// Represents an embed in a message, which can be a website, image, video, text, or none.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Embed {
    /// Represents the website variant for this public enum.
    Website(WebsiteMetadata),
    /// Represents the image variant for this public enum.
    Image(Image),
    /// Represents the video variant for this public enum.
    Video(Video),
    /// Represents the text variant for this public enum.
    Text(Text),
    /// Represents the none variant for this public enum.
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
    /// The title value associated with this embed create.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The human-readable description attached to this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Attachment ID or media reference for the embed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<String>,
    /// The role or embed colour value encoded as an API integer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
}

/// Represents an image value used by the Stoat API models and endpoints.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Image {
    /// The URL value associated with this image.
    pub url: String,

    /// The width value associated with this image.
    pub width: isize,

    /// The height value associated with this image.
    pub height: isize,

    /// The size value associated with this image.
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

/// Represents a video value used by the Stoat API models and endpoints.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Video {
    /// The URL value associated with this video.
    pub url: String,

    /// The width value associated with this video.
    pub width: isize,

    /// The height value associated with this video.
    pub height: isize,
}

/// Represents a text value used by the Stoat API models and endpoints.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Text {
    /// The icon URL value associated with this text.
    pub icon_url: Option<String>,

    /// The URL value associated with this text.
    pub url: Option<String>,

    /// The title value associated with this text.
    pub title: Option<String>,

    /// The human-readable description attached to this resource.
    pub description: Option<String>,

    /// The media value associated with this text.
    pub media: Option<Attachment>,

    /// The role or embed colour value encoded as an API integer.
    pub colour: Option<String>,
}

/// Represents special remote content that can be embedded in a message.
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Special {
    /// Represents the none variant for this public enum.
    None,

    /// Content hint that this contains a GIF
    ///
    /// Use metadata to find video or image to play
    GIF,

    /// Represents the you tube variant for this public enum.
    YouTube {
        /// YouTube video ID.
        id: String,
        /// Optional playback timestamp.
        timestamp: Option<String>,
    },

    /// Represents the lightspeed variant for this public enum.
    Lightspeed {
        /// Lightspeed content type.
        content_type: LightspeedType,
        /// Lightspeed content ID.
        id: String,
    },

    /// Represents the twitch variant for this public enum.
    Twitch {
        /// Twitch content type.
        content_type: TwitchType,
        /// Twitch content ID.
        id: String,
    },
    /// Represents the spotify variant for this public enum.
    Spotify {
        /// Spotify content type.
        content_type: String,
        /// Spotify content ID.
        id: String,
    },

    /// Represents the soundcloud variant for this public enum.
    Soundcloud,

    /// Represents the bandcamp variant for this public enum.
    Bandcamp {
        /// Bandcamp content type.
        content_type: BandcampType,
        /// Bandcamp content ID.
        id: String,
    },
}

/// Represents the supported twitch type variants returned by or sent to the Stoat API.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum TwitchType {
    /// Represents the channel variant for this public enum.
    Channel,
    /// Represents the video variant for this public enum.
    Video,
    /// Represents the clip variant for this public enum.
    Clip,
}

/// Represents the supported lightspeed type variants returned by or sent to the Stoat API.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum LightspeedType {
    /// Represents the channel variant for this public enum.
    Channel,
}

/// Represents the supported bandcamp type variants returned by or sent to the Stoat API.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum BandcampType {
    /// Represents the album variant for this public enum.
    Album,
    /// Represents the track variant for this public enum.
    Track,
}

/// Metadata for a website that can be embedded in a message.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct WebsiteMetadata {
    /// The URL value associated with this website metadata.
    pub url: Option<String>,

    /// The original URL value associated with this website metadata.
    pub original_url: Option<String>,

    /// The special value associated with this website metadata.
    pub special: Option<Special>,

    /// The title value associated with this website metadata.
    pub title: Option<String>,

    /// The human-readable description attached to this resource.
    pub description: Option<String>,

    /// The image value associated with this website metadata.
    pub image: Option<Image>,

    /// The video value associated with this website metadata.
    pub video: Option<Video>,

    /// The site name value associated with this website metadata.
    pub site_name: Option<String>,

    /// The icon URL value associated with this website metadata.
    pub icon_url: Option<String>,

    /// The role or embed colour value encoded as an API integer.
    pub colour: Option<String>,
}
