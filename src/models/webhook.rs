use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{Attachment, Id};

/// Represents a webhook value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Webhook {
    /// The unique ID assigned to this resource by the Stoat API.
    #[serde(rename = "_id")]
    pub id: Id,
    /// The display name or configured name for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The avatar attachment or avatar reference associated with this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    /// The ID of the channel associated with this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<Id>,
    /// The token used to authenticate or execute this API resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a webhook create value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct WebhookCreate {
    /// The display name or configured name for this resource.
    pub name: String,
    /// The avatar attachment or avatar reference associated with this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
}

/// Represents a webhook update value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct WebhookUpdate {
    /// The display name or configured name for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The avatar attachment or avatar reference associated with this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
    /// The list of fields that should be removed from the resource during update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<WebhookFields>,
}

/// Represents the supported webhook fields variants returned by or sent to the Stoat API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum WebhookFields {
    /// Represents the avatar variant for this public enum.
    Avatar,
}

/// Represents a webhook execute value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct WebhookExecute {
    /// The textual content included in this message or payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The username displayed for the user or bot account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The avatar URL to use when presenting this message or webhook execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// The attachments included with this message or webhook payload.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<Id>,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

use crate::{http::HttpClient, KahoResult};

impl Webhook {
    /// Fetch a fresh copy of this webhook.
    pub async fn fetch(&self, http: &HttpClient) -> KahoResult<Self> {
        http.fetch_webhook(&self.id).await
    }

    /// Calls the Stoat API or client internals to edit for this resource.
    pub async fn edit(&self, http: &HttpClient, payload: impl Into<WebhookUpdate>) -> KahoResult<Self> {
        http.edit_webhook(&self.id, payload).await
    }

    /// Calls the Stoat API or client internals to delete for this resource.
    pub async fn delete(&self, http: &HttpClient) -> KahoResult {
        http.delete_webhook(&self.id).await
    }

    /// Execute this webhook when its token is present on the model.
    pub async fn execute(&self, http: &HttpClient, payload: impl Into<WebhookExecute>) -> KahoResult {
        let token = self.token.as_deref().unwrap_or_default();
        http.execute_webhook(&self.id, token, payload).await
    }
}
