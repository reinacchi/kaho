use serde::{Deserialize, Serialize};

/// Represents the fields that can be included in a member object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MemberFields {
    /// Represents the avatar variant for this public enum.
    Avatar,
    /// Represents the nickname variant for this public enum.
    Nickname,
    /// Represents the roles variant for this public enum.
    Roles,
    /// Represents the timeout variant for this public enum.
    Timeout,
}

use crate::models::{Attachment, Id, ServerBan, User};

/// Represents a member value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Member {
    /// The unique ID assigned to the `Member` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The ID of the server associated with the `Member`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Id>,
    /// The nickname value associated with this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The avatar attachment or avatar reference associated with the `Member`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    /// Role IDs assigned to the member.
    #[serde(default)]
    pub roles: Vec<Id>,
    /// The timeout value associated with this member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

/// Represents a member update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MemberUpdate {
    /// The nickname value associated with this member update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// The avatar attachment or avatar reference associated with the `MemberUpdate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
    /// The role IDs or role objects associated with this server resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Id>>,
    /// The timeout value associated with this member update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    /// The list of fields that should be removed from the resource during update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<MemberFields>,
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
    /// The user IDs included in this response or request payload.
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
    /// The user IDs included in this response or request payload.
    #[serde(default)]
    pub users: Vec<User>,
    /// The bans value associated with this server bans.
    #[serde(default)]
    pub bans: Vec<ServerBan>,
}
