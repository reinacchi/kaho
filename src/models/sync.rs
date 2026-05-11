use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a sync settings fetch value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SyncSettingsFetch {
    /// The settings keys requested from the synced settings endpoint.
    #[serde(default)]
    pub keys: Vec<String>,
}

/// Represents a sync settings set value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SyncSettingsSet {
    /// The synced settings key to read or update.
    pub key: String,
    /// The JSON value stored for the selected settings key.
    pub value: Value,
}

/// Represents a sync settings value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SyncSettings {
    /// The synced settings values keyed by their settings names.
    #[serde(flatten)]
    pub values: Value,
}

/// Represents an unreads value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Unreads {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}

/// Represents a push subscription value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PushSubscription {
    /// Additional unmodeled API fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: Value,
}
