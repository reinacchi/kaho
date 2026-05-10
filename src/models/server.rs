use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

use crate::models::{
    attachment::Attachment,
    permission::{OverrideField, Permission},
    Id,
};

/// Represents a role in a server, which defines permissions and attributes for members.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Role {
    /// The color associated with the role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,

    /// Whether the role should be displayed separately in the member list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,

    /// The name of the role.
    pub name: String,

    /// The permissions associated with the role.
    pub permissions: OverrideField,

    /// The rank of the role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,
}

/// Represents the fields that can be included in a role object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoleFields {
    Colour,
}

/// Represents the fields that can be included in a server object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerFields {
    Banner,
    Categories,
    Description,
    Icon,
    SystemMessages,
}

/// Represents a category in a server, which can contain multiple channels.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Category {
    /// Channels in this category.
    pub channels: Vec<Id>,
    /// The ID of the category.
    pub id: Id,
    /// The name of the category.
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Server {
    /// Whether the server has analytics enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,

    /// The banner of the server, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<Attachment>,

    /// The categories in the server.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<Category>,

    /// The channels within the server.
    pub channels: Vec<Id>,

    /// The default permissions for the server.
    pub default_permissions: Permission,

    /// The description of the server.
    pub description: String,

    /// Whether the server is discoverable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,

    /// The flags associated with the server.
    pub flags: Option<ServerFlags>,

    /// The ID of the server.
    #[serde(rename = "_id")]
    pub id: Id,

    /// The icon of the server, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,

    /// The name of the server.
    pub name: String,

    /// Whether the server is NSFW (Not Safe For Work).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,

    /// The owner of the server.
    pub owner: Id,

    /// The roles associated with the server.
    #[serde(default = "HashMap::<Id, Role>::new")]
    pub roles: HashMap<Id, Role>,

    /// The system message channels for the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,
}

///
#[derive(Clone, Debug, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

/// Represents a request to edit an existing server.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ServerEdit {
    #[serde(skip_serializing_if = "Option::is_none")]
    analyrics: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    banner: Option<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    categories: Option<Vec<Category>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discoverable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<ServerFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<ServerFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_messages: Option<SystemMessageChannels>,
}
