use serde::{Deserialize, Serialize};

use crate::models::Id;

/// Represents a stored media object, such as an avatar, icon, or message file.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Attachment {
    /// MIME type of the file (e.g., image/png).
    pub content_type: String,
    /// The filename value associated with this attachment.
    pub filename: String,
    /// The unique ID assigned to this resource by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// Metadata describing the nature of the attachment.
    pub metadata: AttachmentMetadata,
    /// The size value associated with this attachment.
    pub size: usize,
    /// Category tag used to classify the attachment.
    pub tag: AttachmentTag,
}

/// Represents the supported attachment tag variants returned by or sent to the Stoat API.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentTag {
    /// Represents the attachments variant for this public enum.
    Attachments,
    /// Represents the avatars variant for this public enum.
    Avatars,
    /// Represents the banners variant for this public enum.
    Banners,
    /// Represents the backgrounds variant for this public enum.
    Backgrounds,
    /// Represents the icons variant for this public enum.
    Icons,
}

/// Type-specific metadata associated with an attachment.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AttachmentMetadata {
    /// Represents the audio variant for this public enum.
    Audio,
    /// Represents the file variant for this public enum.
    File,
    /// Represents the image variant for this public enum.
    Image { height: usize, width: usize },
    /// Represents the text variant for this public enum.
    Text,
    /// Represents the video variant for this public enum.
    Video { height: usize, width: usize },
}
