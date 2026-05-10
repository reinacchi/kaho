use serde::{Deserialize, Serialize};

use crate::models::Id;

/// Represents a stored media object, such as an avatar, icon, or message file.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Attachment {
    /// MIME type of the file (e.g., image/png).
    pub content_type: String,
    /// Original name of the file.
    pub filename: String,
    /// Unique identifier for the attachment.
    #[serde(rename = "_id")]
    pub id: Id,
    /// Metadata describing the nature of the attachment.
    pub metadata: AttachmentMetadata,
    /// File size in bytes.
    pub size: usize,
    /// Category tag used to classify the attachment.
    pub tag: AttachmentTag,
}

/// Logical category assigned to an attachment.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentTag {
    Attachments,
    Avatars,
    Banners,
    Backgrounds,
    Icons,
}

/// Type-specific metadata associated with an attachment.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AttachmentMetadata {
    /// An audio file.
    Audio,
    /// A generic file.
    File,
    /// An image, with resolution details.
    Image {
        height: usize,
        width: usize,
    },
    /// A plain text file.
    Text,
    /// A video, with resolution details.
    Video {
        height: usize,
        width: usize,
    },
}
