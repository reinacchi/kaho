use serde::{Deserialize, Serialize};

/// Represents a single permission as a bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum PermissionFlag {
    /// Represents the manage channel variant for this public enum.
    ManageChannel        = 1 << 0,
    /// Represents the manage server variant for this public enum.
    ManageServer         = 1 << 1,
    /// Represents the manage permissions variant for this public enum.
    ManagePermissions    = 1 << 2,
    /// Represents the manage role variant for this public enum.
    ManageRole           = 1 << 3,
    /// Represents the manage customisation variant for this public enum.
    ManageCustomisation  = 1 << 4,

    /// Represents the kick members variant for this public enum.
    KickMembers      = 1 << 6,
    /// Represents the ban members variant for this public enum.
    BanMembers       = 1 << 7,
    /// Represents the timeout members variant for this public enum.
    TimeoutMembers   = 1 << 8,
    /// Represents the assign roles variant for this public enum.
    AssignRoles      = 1 << 9,
    /// Represents the change nickname variant for this public enum.
    ChangeNickname   = 1 << 10,
    /// Represents the manage nicknames variant for this public enum.
    ManageNicknames  = 1 << 11,
    /// Represents the change avatar variant for this public enum.
    ChangeAvatar     = 1 << 12,
    /// Represents the remove avatars variant for this public enum.
    RemoveAvatars    = 1 << 13,

    /// Represents the view channel variant for this public enum.
    ViewChannel         = 1 << 20,
    /// Represents the read message history variant for this public enum.
    ReadMessageHistory  = 1 << 21,
    /// Represents the send message variant for this public enum.
    SendMessage         = 1 << 22,
    /// Represents the manage messages variant for this public enum.
    ManageMessages      = 1 << 23,
    /// Represents the manage webhooks variant for this public enum.
    ManageWebhooks      = 1 << 24,
    /// Represents the invite others variant for this public enum.
    InviteOthers        = 1 << 25,
    /// Represents the send embeds variant for this public enum.
    SendEmbeds          = 1 << 26,
    /// Represents the upload files variant for this public enum.
    UploadFiles         = 1 << 27,
    /// Represents the masquerade variant for this public enum.
    Masquerade          = 1 << 28,
    /// Represents the react variant for this public enum.
    React               = 1 << 29,

    /// Represents the connect variant for this public enum.
    Connect         = 1 << 30,
    /// Represents the speak variant for this public enum.
    Speak           = 1 << 31,
    /// Represents the video variant for this public enum.
    Video           = 1 << 32,
    /// Represents the mute members variant for this public enum.
    MuteMembers     = 1 << 33,
    /// Represents the deafen members variant for this public enum.
    DeafenMembers   = 1 << 34,
    /// Represents the move members variant for this public enum.
    MoveMembers     = 1 << 35,
}

/// A set of permissions stored as a bitfield.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Permission(pub u64);

impl Permission {
    /// Return whether the permission set includes `flag`.
    pub fn contains(&self, flag: PermissionFlag) -> bool {
        self.0 & (flag as u64) != 0
    }

    /// Add `flag` to the permission set.
    pub fn insert(&mut self, flag: PermissionFlag) {
        self.0 |= flag as u64;
    }

    /// Remove `flag` from the permission set.
    pub fn remove(&mut self, flag: PermissionFlag) {
        self.0 &= !(flag as u64);
    }

    /// Return `true` when no permissions are set.
    /// Return `true` when no user permissions are set.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

/// Represents user-specific permissions using a bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct UserPermission(pub u64);

/// Represents the supported user permission flag variants returned by or sent to the Stoat API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum UserPermissionFlag {
    /// Represents the access variant for this public enum.
    Access       = 1 << 0,
    /// Represents the view profile variant for this public enum.
    ViewProfile  = 1 << 1,
    /// Represents the send message variant for this public enum.
    SendMessage  = 1 << 2,
    /// Represents the invite variant for this public enum.
    Invite       = 1 << 3,
}

impl UserPermission {
    /// Return whether the user permission set includes `flag`.
    pub fn contains(&self, flag: UserPermissionFlag) -> bool {
        self.0 & (flag as u64) != 0
    }

    /// Add `flag` to the user permission set.
    pub fn insert(&mut self, flag: UserPermissionFlag) {
        self.0 |= flag as u64;
    }

    /// Remove `flag` from the user permission set.
    pub fn remove(&mut self, flag: UserPermissionFlag) {
        self.0 &= !(flag as u64);
    }

    /// Return `true` when no user permissions are set.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

/// Raw representation of permission overrides used in storage.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct OverrideField {
    /// The a value associated with this override field.
    pub a: Permission,
    /// The d value associated with this override field.
    pub d: Permission,
}

/// Represents an override value used by the Stoat API models and endpoints.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Override {
    /// The allow value associated with this override.
    pub allow: Permission,
    /// The deny value associated with this override.
    pub deny: Permission,
}
