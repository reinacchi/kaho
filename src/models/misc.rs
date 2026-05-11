use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{Channel, Id, User};

/// Instance configuration returned by the API root endpoint.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct InstanceConfig {
    /// Raw configuration payload. Kept flexible because self-hosted Stoat
    /// instances may expose optional sections not present on every server.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a safety report create value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SafetyReportCreate {
    /// The textual content included in this message or payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Id>,
    /// The ID of the user associated with the `SafetyReportCreate`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Id>,
    /// The reason value associated with this safety report create.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents an onboarding hello value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OnboardingHello {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents an onboarding complete value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OnboardingComplete {
    /// The username displayed for the user or bot account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The captcha response used to satisfy verification requirements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha: Option<String>,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a direct message open value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DirectMessageOpen {
    /// The ID of the channel associated with the `DirectMessageOpen`.
    pub channel: Channel,
    /// The user IDs included in this response or request payload.
    #[serde(default)]
    pub users: Vec<User>,
}

/// Generic success response for endpoints that may return an object on some
/// deployments and no body on others.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ApiAck {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}
