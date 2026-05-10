use serde::{Deserialize, Serialize};

/// Represents the fields that can be included in a member object.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MemberFields {
    /// Member avatar.
    Avatar,
    /// Member nickname.
    Nickname,
    /// Member role assignments.
    Roles,
    /// Member timeout.
    Timeout,
}
