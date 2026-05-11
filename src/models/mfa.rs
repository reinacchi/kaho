use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a MFA ticket payload value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MfaTicketPayload {
    /// The ticket value associated with this MFA ticket payload.
    pub ticket: String,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a MFA status value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MfaStatus {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a MFA methods value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MfaMethods {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a MFA recovery payload value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MfaRecoveryPayload {
    /// The password value supplied for account authentication or confirmation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// The code value associated with this MFA recovery payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a MFA TOTP payload value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MfaTotpPayload {
    /// The password value supplied for account authentication or confirmation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// The code value associated with this MFA TOTP payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a MFA response value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MfaResponse {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}
