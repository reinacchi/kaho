use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    http::HttpClient,
    models::{Attachment, Id},
    KahoResult,
};

/// Represents a user.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
    /// Replacement display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The status of the user.
    pub status: Option<UserStatus>,
    /// Relationships associated with the user.
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

    /// Change the current user's username.
    ///
    /// This endpoint only applies to the authenticated user; the method is
    /// available on `User` for ergonomic use when `self` is the current user.
    pub async fn change_username(&self, http: &HttpClient, username: impl Into<String>, password: impl Into<String>) -> KahoResult<Self> {
        http.change_username(ChangeUsername {
            username: username.into(),
            password: password.into(),
        })
        .await
    }

    /// Fetch this user's flags.
    pub async fn flags(&self, http: &HttpClient) -> KahoResult<FlagResponse> {
        http.fetch_user_flags(&self.id).await
    }

    /// Fetch this user's default avatar image.
    pub async fn default_avatar(&self, http: &HttpClient) -> KahoResult<DefaultAvatar> {
        http.fetch_default_avatar(&self.id).await
    }

    /// Fetch this user's profile.
    pub async fn profile(&self, http: &HttpClient) -> KahoResult<UserProfile> {
        http.fetch_user_profile(&self.id).await
    }

    /// Fetch mutual relationships shared with this user.
    pub async fn mutual(&self, http: &HttpClient) -> KahoResult<MutualResponse> {
        http.fetch_mutual_relationships(&self.id).await
    }

    /// Accept this user's incoming friend request.
    pub async fn accept_friend_request(&self, http: &HttpClient) -> KahoResult<Self> {
        http.accept_friend_request(&self.id).await
    }

    /// Remove this user as a friend, or deny their request.
    pub async fn remove_friend(&self, http: &HttpClient) -> KahoResult<Self> {
        http.remove_friend(&self.id).await
    }

    /// Block this user.
    pub async fn block(&self, http: &HttpClient) -> KahoResult<Self> {
        http.block_user(&self.id).await
    }

    /// Unblock this user.
    pub async fn unblock(&self, http: &HttpClient) -> KahoResult<Self> {
        http.unblock_user(&self.id).await
    }
}

/// User status text and presence.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserStatus {
    /// Custom status text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Current presence string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<String>,
}

/// Known user presence states.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Presence {
    /// The user is online.
    Online,
    /// The user is invisible.
    Invisible,
    /// The user is focusing.
    Focus,
    /// The user is idle.
    Idle,
    /// The user is busy.
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

/// Payload for updating a user.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UserUpdate {
    /// Replacement user status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    /// Replacement profile data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfileUpdate>,
    /// Replacement avatar attachment ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
    /// Replacement display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// User field to remove.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<UserFields>,
    /// Replacement badge bitfield.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<u32>,
    /// Replacement user flag bitfield.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
}

/// Payload for updating a user profile.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UserProfileUpdate {
    /// Replacement profile content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Replacement profile background attachment ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Id>,
}

/// Relationship state between the current user and another user.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserRelationship {
    /// ID of the related user.
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Id,
    /// Relationship status with that user.
    #[serde(default)]
    pub status: RelationshipStatus,
}

/// Relationship state with another user.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum RelationshipStatus {
    /// No relationship.
    #[default]
    None,
    /// Regular user relationship.
    User,
    /// Friend relationship.
    Friend,
    /// Outgoing friend request.
    Outgoing,
    /// Incoming friend request.
    Incoming,
    /// The user is blocked.
    Blocked,
    /// The user has blocked the current user.
    BlockedOther,
}

/// Helper trait for checking relationship status in a relationship list.
pub trait CheckRelationship {
    /// Return the relationship status for `user`.
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

/// User fields that can be removed or edited by update payloads.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum UserFields {
    /// User avatar.
    Avatar,
    /// Status text.
    StatusText,
    /// Status presence.
    StatusPresence,
    /// Profile content.
    ProfileContent,
    /// Profile background.
    ProfileBackground,
    /// Display name.
    DisplayName,
}

bitflags! {
    /// Badge flags attached to a user.
    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
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
    /// Account flags attached to a user.
    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
    #[serde(transparent)]
    pub struct UserFlags: u32 {
        const Suspended = 1;
        const Deleted = 2;
        const Banned = 4;
        const Spam = 8;
    }
}

/// Response containing a user flag bitfield.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FlagResponse {
    /// Raw user flags returned by the API.
    pub flags: i32,
}

/// Mutual users and servers shared with another user.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MutualResponse {
    /// Mutual user IDs.
    pub users: Vec<String>,
    /// Mutual server IDs.
    pub servers: Vec<String>,
    /// Mutual group or DM channel IDs.
    #[serde(default)]
    pub channels: Vec<String>,
}

/// Public bot ownership information.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotInformation {
    /// ID of the bot owner.
    pub owner: String,
}

/// Request payload for sending a friend request.
#[derive(Clone, Debug, Serialize)]
pub struct SendFriendRequest {
    /// Username to send a friend request to.
    pub username: String,
}


/// Payload for changing the authenticated user's username.
#[derive(Clone, Debug, Serialize)]
pub struct ChangeUsername {
    /// New username.
    pub username: String,
    /// Current account password.
    pub password: String,
}

/// User profile returned by the profile endpoint.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserProfile {
    /// Profile content / bio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Profile background attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Attachment>,
}

/// Default avatar image returned by the API.
#[derive(Clone, Debug, PartialEq)]
pub struct DefaultAvatar {
    /// Raw PNG image bytes.
    pub bytes: Vec<u8>,
}
