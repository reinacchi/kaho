use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{
    attachment::Attachment,
    permission::{OverrideField, Permission},
    Id,
};

/// Represents a role in a server, which defines permissions and attributes for members.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Role {
    /// The colour associated with the role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,

    /// Whether the role should be displayed separately in the member list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,

    /// The display name or configured name for the `Role`.
    pub name: String,

    /// The permissions associated with the role.
    pub permissions: OverrideField,

    /// The ordering rank used when sorting roles or members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
}

/// Represents the fields that can be included in a role object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoleFields {
    /// Represents the colour variant for this public enum.
    Colour,
}

/// Represents the fields that can be included in a server object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerFields {
    /// Represents the banner variant for this public enum.
    Banner,
    /// Represents the categories variant for this public enum.
    Categories,
    /// Represents the description variant for this public enum.
    Description,
    /// Represents the icon variant for this public enum.
    Icon,
    /// Represents the system messages variant for this public enum.
    SystemMessages,
}

/// Represents a category in a server, which can contain multiple channels.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Category {
    /// The channels value associated with this category.
    pub channels: Vec<Id>,
    /// The unique ID assigned to the `Category` by the Stoat API.
    pub id: Id,
    /// The title value associated with this category.
    pub title: String,
}

/// Holds the channel IDs used by the system to send automatic messages
/// when certain member-related events occur on a server.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct SystemMessageChannels {
    /// Channel where a message is posted when someone is banned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_banned: Option<Id>,

    /// Channel where a message is sent when someone joins the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_joined: Option<Id>,

    /// Channel where a message is posted when someone is kicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_kicked: Option<Id>,

    /// Channel where a message is sent when someone leaves the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_left: Option<Id>,
}

bitflags! {
    /// Represents the flags associated with a server.
    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
    pub struct ServerFlags: u32 {
        const VerifiedServer = 1;
        const OfficialServer = 2;
    }
}

/// Represents a server value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Server {
    /// Whether the server has analytics enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,

    /// The banner of the server, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<Attachment>,

    /// The categories value associated with this server.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<Category>,

    /// The channels value associated with this server.
    pub channels: Vec<Id>,

    /// The default permissions for the server.
    pub default_permissions: Permission,

    /// The human-readable description attached to the `Server`.
    pub description: String,

    /// The discoverable value associated with this server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,

    /// The flags associated with the server.
    pub flags: Option<ServerFlags>,

    /// The unique ID assigned to the `Server` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,

    /// The icon of the server, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,

    /// The display name or configured name for the `Server`.
    pub name: String,

    /// Whether the server is NSFW (Not Safe For Work).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,

    /// The ID of the user or account that owns the `Server`.
    pub owner: Id,

    /// The roles associated with the server.
    #[serde(default = "HashMap::<Id, Role>::new")]
    pub roles: HashMap<Id, Role>,

    /// The system message channels for the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,
}

/// Represents a server ban value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ServerBan {
    /// The ID of the user who is banned.
    #[serde(rename = "_id")]
    pub user: Id,

    /// The reason for the ban, if provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Represents a request to create a new server.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ServerCreate {
    /// The human-readable description attached to the `ServerCreate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The display name or configured name for the `ServerCreate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the server should be marked as NSFW.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/// Represents a request to edit an existing server.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ServerEdit {
    /// The analytics value associated with this server edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,
    /// The banner value associated with this server edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<Attachment>,
    /// The categories value associated with this server edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<Category>>,
    /// The human-readable description attached to the `ServerEdit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The discoverable value associated with this server edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,
    /// The flags value associated with this server edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<ServerFlags>,
    /// The icon attachment or icon reference associated with the `ServerEdit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    /// The display name or configured name for the `ServerEdit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Field to remove from the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<ServerFields>,
    /// The system messages value associated with this server edit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,
}

use crate::{
    http::HttpClient,
    models::{
        BanCreate, Channel, ChannelCreate, FetchMembersQuery, Invite, Member, MemberList,
        MemberUpdate, MembersExperimentalQuery, ServerBans,
    },
    KahoResult,
};

/// Represents a role create value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RoleCreate {
    /// The display name or configured name for the `RoleCreate`.
    pub name: String,
    /// The ordering rank used when sorting roles or members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
}

/// Response returned by create-role endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RoleCreateResponse {
    /// The unique ID assigned to the `RoleCreateResponse` by the Stoat API.
    pub id: Id,
    /// The role value associated with this role create response.
    pub role: Role,
}

/// Represents a role update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RoleUpdate {
    /// The display name or configured name for the `RoleUpdate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The role or embed colour value encoded as an API integer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
    /// Whether members with this role should be visually separated in member lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    /// The ordering rank used when sorting roles or members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
    /// The permission bitfield applied to this role or resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<OverrideField>,
    /// The list of fields that should be removed from the resource during update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<RoleFields>,
}

/// Represents a role ranks update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RoleRanksUpdate {
    /// The ordered role IDs used to update server role ranks.
    pub ranks: Vec<Id>,
}

impl Server {
    /// Fetch a fresh copy of this server.
    pub async fn fetch(&self, http: &HttpClient) -> KahoResult<Self> {
        http.fetch_server(&self.id).await
    }

    /// Edit the `Server`.
    pub async fn edit(
        &self,
        http: &HttpClient,
        payload: impl Into<ServerEdit>,
    ) -> KahoResult<Self> {
        http.edit_server(&self.id, payload).await
    }

    /// Delete the `Server`.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_server(&self.id).await
    }

    /// Acknowledge the `Server`.
    pub async fn acknowledge(&self, http: &HttpClient) -> KahoResult {
        http.acknowledge_server(&self.id).await
    }

    /// Create a channel inside this server.
    pub async fn create_channel(
        &self,
        http: &HttpClient,
        payload: impl Into<ChannelCreate>,
    ) -> KahoResult<Channel> {
        http.create_server_channel(&self.id, payload).await
    }

    /// Fetch members for the `Server`.
    pub async fn members(
        &self,
        http: &HttpClient,
        query: impl Into<Option<FetchMembersQuery>>,
    ) -> KahoResult<MemberList> {
        http.fetch_server_members(&self.id, query).await
    }

    /// Fetch one member from this server.
    pub async fn member(&self, http: &HttpClient, user_id: &str) -> KahoResult<Member> {
        http.fetch_server_member(&self.id, user_id).await
    }

    /// Kick a member from this server.
    pub async fn kick_member(&self, http: &HttpClient, user_id: &str) -> KahoResult {
        http.kick_server_member(&self.id, user_id).await
    }

    /// Edit a member in this server.
    pub async fn edit_member(
        &self,
        http: &HttpClient,
        user_id: &str,
        payload: impl Into<MemberUpdate>,
    ) -> KahoResult<Member> {
        http.edit_server_member(&self.id, user_id, payload).await
    }

    /// Query members using the experimental member query endpoint.
    pub async fn query_members(
        &self,
        http: &HttpClient,
        payload: impl Into<MembersExperimentalQuery>,
    ) -> KahoResult<MemberList> {
        http.query_server_members(&self.id, payload).await
    }

    /// Ban a user from this server.
    pub async fn ban_user(
        &self,
        http: &HttpClient,
        user_id: &str,
        payload: impl Into<BanCreate>,
    ) -> KahoResult {
        http.ban_user(&self.id, user_id, payload).await
    }

    /// Unban a user from this server.
    pub async fn unban_user(&self, http: &HttpClient, user_id: &str) -> KahoResult {
        http.unban_user(&self.id, user_id).await
    }

    /// Fetch bans for the `Server`.
    pub async fn bans(&self, http: &HttpClient) -> KahoResult<ServerBans> {
        http.fetch_server_bans(&self.id).await
    }

    /// Fetch invites for the `Server`.
    pub async fn invites(&self, http: &HttpClient) -> KahoResult<Vec<Invite>> {
        http.fetch_server_invites(&self.id).await
    }

    /// Create a role in this server.
    pub async fn create_role(
        &self,
        http: &HttpClient,
        payload: impl Into<RoleCreate>,
    ) -> KahoResult<RoleCreateResponse> {
        http.create_server_role(&self.id, payload).await
    }

    /// Fetch a role from this server.
    pub async fn role(&self, http: &HttpClient, role_id: &str) -> KahoResult<Role> {
        http.fetch_server_role(&self.id, role_id).await
    }

    /// Delete a role from this server.
    pub async fn delete_role(&self, http: &HttpClient, role_id: &str) -> KahoResult {
        http.delete_server_role(&self.id, role_id).await
    }

    /// Edit a role in this server.
    pub async fn edit_role(
        &self,
        http: &HttpClient,
        role_id: &str,
        payload: impl Into<RoleUpdate>,
    ) -> KahoResult<Role> {
        http.edit_server_role(&self.id, role_id, payload).await
    }

    /// Set permissions for a role in this server.
    pub async fn set_role_permissions(
        &self,
        http: &HttpClient,
        role_id: &str,
        payload: OverrideField,
    ) -> KahoResult {
        http.set_server_permissions(&self.id, role_id, payload)
            .await
    }

    /// Set default permissions for this server.
    pub async fn set_default_permissions(
        &self,
        http: &HttpClient,
        payload: OverrideField,
    ) -> KahoResult {
        http.set_server_default_permissions(&self.id, payload).await
    }

    /// Reorder role ranks in this server.
    pub async fn set_role_ranks(
        &self,
        http: &HttpClient,
        payload: impl Into<RoleRanksUpdate>,
    ) -> KahoResult {
        http.set_server_role_ranks(&self.id, payload).await
    }
}
