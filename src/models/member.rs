use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{Attachment, Id, Role, ServerBan, User};

/// Composite identifier for a server member.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MemberId {
    /// Server ID containing the member.
    pub server: Id,
    /// User ID represented by this member.
    pub user: Id,
}

/// Represents the fields that can be removed from a member object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MemberFields {
    /// Represents the nickname variant for this public enum.
    Nickname,
    /// Represents the pronouns variant for this public enum.
    Pronouns,
    /// Represents the avatar variant for this public enum.
    Avatar,
    /// Represents the roles variant for this public enum.
    Roles,
    /// Represents the timeout variant for this public enum.
    Timeout,
    /// Represents the can receive variant for this public enum.
    CanReceive,
    /// Represents the can publish variant for this public enum.
    CanPublish,
    /// Represents the joined at variant for this public enum.
    JoinedAt,
    /// Represents the voice channel variant for this public enum.
    VoiceChannel,
}

/// Represents a member value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Member {
    /// Composite server/user ID assigned to the member by the Stoat API.
    #[serde(rename = "_id")]
    pub id: MemberId,
    /// ISO-8601 timestamp at which this user joined the server.
    pub joined_at: String,
    /// The nickname value associated with this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The pronouns value associated with this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    /// The avatar attachment or avatar reference associated with the member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    /// Role IDs assigned to the member.
    #[serde(default)]
    pub roles: Vec<Id>,
    /// The timeout value associated with this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    /// Whether the member may publish voice server-wide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_publish: Option<bool>,
    /// Whether the member may receive voice server-wide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_receive: Option<bool>,
}

/// Extended member response returned when role expansion is requested.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemberWithRoles {
    /// Member object returned by Stoat.
    pub member: Member,
    /// Roles referenced by the member, keyed by role ID.
    pub roles: HashMap<Id, Role>,
}

/// Response returned by the fetch-member endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MemberResponse {
    /// Plain member response.
    Member(Member),
    /// Member response with expanded roles.
    WithRoles(MemberWithRoles),
}

impl MemberResponse {
    /// Consume the response and return its member object.
    pub fn into_member(self) -> Member {
        match self {
            Self::Member(member) => member,
            Self::WithRoles(response) => response.member,
        }
    }

    /// Return the expanded roles when they were requested.
    pub fn roles(&self) -> Option<&HashMap<Id, Role>> {
        match self {
            Self::Member(_) => None,
            Self::WithRoles(response) => Some(&response.roles),
        }
    }
}

/// Represents a member update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MemberUpdate {
    /// The nickname value associated with this member update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The pronouns value associated with this member update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    /// The avatar attachment ID associated with this member update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
    /// Role IDs assigned to this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Id>>,
    /// The timeout value associated with this member update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    /// Whether the member may publish voice server-wide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_publish: Option<bool>,
    /// Whether the member may receive voice server-wide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_receive: Option<bool>,
    /// Voice channel to move the member to when they are already in voice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_channel: Option<Id>,
    /// Fields that should be removed from the member during update.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove: Vec<MemberFields>,
}

/// Represents a fetch members query value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct FetchMembersQuery {
    /// The exclude offline value associated with this fetch members query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_offline: Option<bool>,
}

/// Represents a members experimental query value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MembersExperimentalQuery {
    /// The query value associated with this members experimental query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// The limit value associated with this members experimental query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Represents a member list value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MemberList {
    /// The members returned for this server or group query.
    #[serde(default)]
    pub members: Vec<Member>,
    /// The users included in this response payload.
    #[serde(default)]
    pub users: Vec<User>,
}

/// Represents a ban create value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BanCreate {
    /// The reason value associated with this ban create.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Represents a server bans value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ServerBans {
    /// The users included in this response payload.
    #[serde(default)]
    pub users: Vec<User>,
    /// The bans value associated with this server bans response.
    #[serde(default)]
    pub bans: Vec<ServerBan>,
}
