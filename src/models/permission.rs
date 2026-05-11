use serde::{Deserialize, Serialize};

/// Represents a single permission as a bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum PermissionFlag {
    /// Manage channels.
    ManageChannel        = 1 << 0,
    /// Manage server settings.
    ManageServer         = 1 << 1,
    /// Manage permission overrides.
    ManagePermissions    = 1 << 2,
    /// Manage roles.
    ManageRole           = 1 << 3,
    /// Manage server customisation.
    ManageCustomisation  = 1 << 4,

    /// Kick members.
    KickMembers      = 1 << 6,
    /// Ban members.
    BanMembers       = 1 << 7,
    /// Timeout members.
    TimeoutMembers   = 1 << 8,
    /// Assign roles to members.
    AssignRoles      = 1 << 9,
    /// Change own nickname.
    ChangeNickname   = 1 << 10,
    /// Manage other members' nicknames.
    ManageNicknames  = 1 << 11,
    /// Change own avatar.
    ChangeAvatar     = 1 << 12,
    /// Remove members' avatars.
    RemoveAvatars    = 1 << 13,

    /// View a channel.
    ViewChannel         = 1 << 20,
    /// Read previous messages.
    ReadMessageHistory  = 1 << 21,
    /// Send messages.
    SendMessage         = 1 << 22,
    /// Manage messages.
    ManageMessages      = 1 << 23,
    /// Manage webhooks.
    ManageWebhooks      = 1 << 24,
    /// Invite other users.
    InviteOthers        = 1 << 25,
    /// Send embeds.
    SendEmbeds          = 1 << 26,
    /// Upload files.
    UploadFiles         = 1 << 27,
    /// Masquerade message display details.
    Masquerade          = 1 << 28,
    /// React to messages.
    React               = 1 << 29,

    /// Connect to voice.
    Connect         = 1 << 30,
    /// Speak in voice.
    Speak           = 1 << 31,
    /// Use video in voice.
    Video           = 1 << 32,
    /// Mute members in voice.
    MuteMembers     = 1 << 33,
    /// Deafen members in voice.
    DeafenMembers   = 1 << 34,
    /// Move members between voice channels.
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

/// User relationship permission flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum UserPermissionFlag {
    /// Access the user.
    Access       = 1 << 0,
    /// View the user profile.
    ViewProfile  = 1 << 1,
    /// Send the user a message.
    SendMessage  = 1 << 2,
    /// Invite the user.
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
