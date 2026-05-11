use serde::{Deserialize, Serialize};

use crate::{http::HttpClient, models::{Id, Server, Channel, User}, KahoResult};

/// Invite object returned by invite endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Invite {
    /// Invite code.
    #[serde(rename = "_id")]
    pub id: Id,
    /// Server referenced by the invite, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,
    /// Channel referenced by the invite, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    /// User who created the invite, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,
}

impl Invite {
    /// Accept this invite.
    pub async fn accept(&self, http: &HttpClient) -> KahoResult<InviteJoinResponse> {
        http.accept_invite(&self.id).await
    }

    /// Delete this invite.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_invite(&self.id).await
    }
}

/// Response returned after accepting an invite.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InviteJoinResponse {
    /// Joined server ID, when the invite points at a server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Id>,
    /// Joined channel ID, when returned by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Id>,
}
