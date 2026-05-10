use serde::{Deserialize, Serialize};

/// Represents a single permission as a bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum PermissionFlag {
    // Generic permissions
    ManageChannel        = 1 << 0,
    ManageServer         = 1 << 1,
    ManagePermissions    = 1 << 2,
    ManageRole           = 1 << 3,
    ManageCustomisation  = 1 << 4,

    // Member permissions
    KickMembers      = 1 << 6,
    BanMembers       = 1 << 7,
    TimeoutMembers   = 1 << 8,
    AssignRoles      = 1 << 9,
    ChangeNickname   = 1 << 10,
    ManageNicknames  = 1 << 11,
    ChangeAvatar     = 1 << 12,
    RemoveAvatars    = 1 << 13,

    // Channel permissions
    ViewChannel         = 1 << 20,
    ReadMessageHistory  = 1 << 21,
    SendMessage         = 1 << 22,
    ManageMessages      = 1 << 23,
    ManageWebhooks      = 1 << 24,
    InviteOthers        = 1 << 25,
    SendEmbeds          = 1 << 26,
    UploadFiles         = 1 << 27,
    Masquerade          = 1 << 28,
    React               = 1 << 29,

    // Voice permissions
    Connect         = 1 << 30,
    Speak           = 1 << 31,
    Video           = 1 << 32,
    MuteMembers     = 1 << 33,
    DeafenMembers   = 1 << 34,
    MoveMembers     = 1 << 35,
}

/// A set of permissions stored as a bitfield.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Permission(pub u64);

impl Permission {
    pub fn contains(&self, flag: PermissionFlag) -> bool {
        self.0 & (flag as u64) != 0
    }

    pub fn insert(&mut self, flag: PermissionFlag) {
        self.0 |= flag as u64;
    }

    pub fn remove(&mut self, flag: PermissionFlag) {
        self.0 &= !(flag as u64);
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

/// Represents user-specific permissions using a bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct UserPermission(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum UserPermissionFlag {
    Access       = 1 << 0,
    ViewProfile  = 1 << 1,
    SendMessage  = 1 << 2,
    Invite       = 1 << 3,
}

impl UserPermission {
    pub fn contains(&self, flag: UserPermissionFlag) -> bool {
        self.0 & (flag as u64) != 0
    }

    pub fn insert(&mut self, flag: UserPermissionFlag) {
        self.0 |= flag as u64;
    }

    pub fn remove(&mut self, flag: UserPermissionFlag) {
        self.0 &= !(flag as u64);
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

/// Raw representation of permission overrides used in storage.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OverrideField {
    /// Bits for allowed permissions.
    pub a: Permission,
    /// Bits for denied permissions.
    pub d: Permission,
}

/// Processed permission override model.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Override {
    /// Permissions granted explicitly.
    pub allow: Permission,
    /// Permissions explicitly denied.
    pub deny: Permission,
}
