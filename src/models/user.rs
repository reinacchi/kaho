use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    http::HttpClient,
    models::{Attachment, Channel, Id, SafetyReportCreate},
    KahoResult,
};

/// Represents an user value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct User {
    /// The unique ID assigned to this resource by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The username displayed for the user or bot account.
    pub username: String,
    /// The avatar attachment or avatar reference associated with this resource.
    pub avatar: Option<Attachment>,
    /// The discriminator value associated with this user.
    #[serde(default)]
    pub discriminator: String,
    /// The display name of the user.
    /// Replacement display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The status value associated with this user.
    pub status: Option<UserStatus>,
    /// The relations value associated with this user.
    #[serde(default)]
    pub relations: Vec<UserRelationship>,
    /// Whether the user is online or not.
    #[serde(default)]
    pub online: bool,
    /// The badges value associated with this user.
    #[serde(default)]
    pub badges: UserBadges,
    /// The flags value associated with this user.
    #[serde(default)]
    pub flags: UserFlags,
}

impl User {
    /// Calls the Stoat API or client internals to edit for this resource.
    pub async fn edit(
        &self,
        http: &HttpClient,
        payload: impl Into<UserUpdate>,
    ) -> KahoResult<Self> {
        http.edit_user(&self.id, payload.into()).await
    }

    /// Change the current user's username.
    ///
    /// This endpoint only applies to the authenticated user; the method is
    /// available on `User` for ergonomic use when `self` is the current user.
    pub async fn change_username(
        &self,
        http: &HttpClient,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> KahoResult<Self> {
        http.change_username(ChangeUsername {
            username: username.into(),
            password: password.into(),
        })
        .await
    }

    /// Open a direct message channel with this user.
    pub async fn open_dm(&self, http: &HttpClient) -> KahoResult<Channel> {
        http.open_direct_message(&self.id).await
    }

    /// Submit a safety report concerning this user.
    pub async fn report(&self, http: &HttpClient, reason: impl Into<String>) -> KahoResult {
        http.report_safety(SafetyReportCreate {
            user: Some(self.id.clone()),
            reason: Some(reason.into()),
            ..Default::default()
        })
        .await
    }

    /// Calls the Stoat API or client internals to flags for this resource.
    pub async fn flags(&self, http: &HttpClient) -> KahoResult<FlagResponse> {
        http.fetch_user_flags(&self.id).await
    }

    /// Fetch this user's default avatar image.
    pub async fn default_avatar(&self, http: &HttpClient) -> KahoResult<DefaultAvatar> {
        http.fetch_default_avatar(&self.id).await
    }

    /// Calls the Stoat API or client internals to profile for this resource.
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

    /// Calls the Stoat API or client internals to block for this resource.
    pub async fn block(&self, http: &HttpClient) -> KahoResult<Self> {
        http.block_user(&self.id).await
    }

    /// Calls the Stoat API or client internals to unblock for this resource.
    pub async fn unblock(&self, http: &HttpClient) -> KahoResult<Self> {
        http.unblock_user(&self.id).await
    }
}

/// Represents an user status value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserStatus {
    /// The text value associated with this user status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The presence value associated with this user status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<String>,
}

/// Represents the supported presence variants returned by or sent to the Stoat API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Presence {
    /// Represents the online variant for this public enum.
    Online,
    /// Represents the invisible variant for this public enum.
    Invisible,
    /// Represents the focus variant for this public enum.
    Focus,
    /// Represents the idle variant for this public enum.
    Idle,
    /// Represents the busy variant for this public enum.
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

/// Represents an user update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UserUpdate {
    /// The status value associated with this user update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    /// The profile value associated with this user update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfileUpdate>,
    /// The avatar attachment or avatar reference associated with this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
    /// The display name value associated with this user update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The list of fields that should be removed from the resource during update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<UserFields>,
    /// The badges value associated with this user update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<u32>,
    /// The flags value associated with this user update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
}

/// Represents an user profile update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UserProfileUpdate {
    /// The textual content included in this message or payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The background value associated with this user profile update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Id>,
}

/// Relationship state between the current user and another user.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserRelationship {
    /// The unique ID assigned to this resource by the Stoat API.
    #[serde(rename = "_id")]
    #[serde(default)]
    pub id: Id,
    /// The status value associated with this user relationship.
    #[serde(default)]
    pub status: RelationshipStatus,
}

/// Represents the supported relationship status variants returned by or sent to the Stoat API.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum RelationshipStatus {
    /// Represents the none variant for this public enum.
    #[default]
    None,
    /// Represents the user variant for this public enum.
    User,
    /// Represents the friend variant for this public enum.
    Friend,
    /// Represents the outgoing variant for this public enum.
    Outgoing,
    /// Represents the incoming variant for this public enum.
    Incoming,
    /// Represents the blocked variant for this public enum.
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
    /// Represents the avatar variant for this public enum.
    Avatar,
    /// Represents the status text variant for this public enum.
    StatusText,
    /// Represents the status presence variant for this public enum.
    StatusPresence,
    /// Represents the profile content variant for this public enum.
    ProfileContent,
    /// Represents the profile background variant for this public enum.
    ProfileBackground,
    /// Represents the display name variant for this public enum.
    DisplayName,
}

bitflags! {
    /// Represents an user badges value used by the Stoat API models and endpoints.
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
    /// Represents an user flags value used by the Stoat API models and endpoints.
    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
    #[serde(transparent)]
    pub struct UserFlags: u32 {
        const Suspended = 1;
        const Deleted = 2;
        const Banned = 4;
        const Spam = 8;
    }
}

/// Represents a flag response value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FlagResponse {
    /// Raw user flags returned by the API.
    pub flags: i32,
}

/// Mutual users and servers shared with another user.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MutualResponse {
    /// The user IDs included in this response or request payload.
    pub users: Vec<String>,
    /// The servers value associated with this mutual response.
    pub servers: Vec<String>,
    /// The channels value associated with this mutual response.
    #[serde(default)]
    pub channels: Vec<String>,
}

/// Represents a bot information value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BotInformation {
    /// The ID of the user or account that owns this resource.
    pub owner: String,
}

/// Represents a send friend request value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct SendFriendRequest {
    /// The username displayed for the user or bot account.
    pub username: String,
}

/// Payload for changing the authenticated user's username.
#[derive(Clone, Debug, Serialize)]
pub struct ChangeUsername {
    /// The username displayed for the user or bot account.
    pub username: String,
    /// The password value supplied for account authentication or confirmation.
    pub password: String,
}

/// User profile returned by the profile endpoint.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UserProfile {
    /// The textual content included in this message or payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The background value associated with this user profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Attachment>,
}

/// Default avatar image returned by the API.
#[derive(Clone, Debug, PartialEq)]
pub struct DefaultAvatar {
    /// The raw bytes for the image or binary payload being uploaded.
    pub bytes: Vec<u8>,
}
