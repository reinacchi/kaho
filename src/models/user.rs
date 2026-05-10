use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    http::HttpClient,
    models::{Attachment, Id},
    KahoResult,
};

/// Represents a user.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct User {
    /// The ID of the user.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The username.
    pub username: String,
    /// The avatar of the user.
    pub avatar: Option<Attachment>,
    /// The discriminator of the user.
    #[serde(default)]
    pub discriminator: String,
    /// The display name of the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The status of the user.
    pub status: Option<UserStatus>,
    #[serde(default)]
    pub relations: Vec<UserRelationship>,
    /// Whether the user is online or not.
    #[serde(default)]
    pub online: bool,
    /// The badges of the user.
    #[serde(default)]
    pub badges: UserBadges,
    /// The flags of the user.
    #[serde(default)]
    pub flags: UserFlags,
}

impl User {
    /// Edit this user.
    pub async fn edit(&self, http: &HttpClient, payload: impl Into<UserUpdate>) -> KahoResult<Self> {
        http.edit_user(&self.id, payload.into()).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Presence {
    Online,
    Invisible,
    Focus,
    Idle,
    Busy,
}

impl fmt::Display for Presence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Presence::Online => "Online",
            Presence::Invisible => "Invisible",
            Presence::Focus => "Focus",
            Presence::Idle => "Idle",
            Presence::Busy => "Busy",
        };

        formatter.write_str(value)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UserUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfileUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<UserFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UserProfileUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Id>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserRelationship {
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Id,
    #[serde(default)]
    pub status: RelationshipStatus,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum RelationshipStatus {
    #[default]
    None,
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
}

pub trait CheckRelationship {
    fn with(&self, user: &str) -> RelationshipStatus;
}

impl CheckRelationship for Vec<UserRelationship> {
    fn with(&self, user: &str) -> RelationshipStatus {
        for entry in self {
            if entry.id == user {
                return entry.status.clone();
            }
        }

        RelationshipStatus::None
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum UserFields {
    Avatar,
    StatusText,
    StatusPresence,
    ProfileContent,
    ProfileBackground,
    DisplayName,
}

bitflags! {
    #[derive(Clone, Debug, PartialEq, Deserialize, Default)]
    #[serde(transparent)]
    pub struct UserBadges: u32 {
        const Developer = 1;
        const Translator = 2;
        const Supporter = 4;
        const ResponsibleDisclosure = 8;
        const Founder = 16;
        const PlatformModeration = 32;
        const ActiveSupporter = 64;
        const Paw = 128;
        const EarlyAdopter = 256;
        const ReservedRelevantJokeBadge1 = 512;
        const ReservedRelevantJokeBadge2 = 1024;
    }
}

bitflags! {
    #[derive(Clone, Debug, PartialEq, Deserialize, Default)]
    #[serde(transparent)]
    pub struct UserFlags: u32 {
        const Suspended = 1;
        const Deleted = 2;
        const Banned = 4;
        const Spam = 8;
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FlagResponse {
    pub flags: i32,
}

pub struct MutualResponse {
    pub users: Vec<String>,
    pub servers: Vec<String>,
}

pub struct BotInformation {
    pub owner: String,
}

pub struct SendFriendRequest {
    pub username: String,
}
