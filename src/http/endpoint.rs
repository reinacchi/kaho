/// Represents the supported endpoint variants returned by or sent to the Stoat API.
#[derive(Debug, Clone)]
pub enum Endpoint {
    // Account-related
    /// Represents the account variant for this public enum.
    Account,
    /// Represents the account change email variant for this public enum.
    AccountChangeEmail,
    /// Represents the account change password variant for this public enum.
    AccountChangePassword,
    /// Represents the account create variant for this public enum.
    AccountCreate,
    /// Represents the account delete variant for this public enum.
    AccountDelete,
    /// Represents the account disable variant for this public enum.
    AccountDisable,
    /// Represents the account reset password variant for this public enum.
    AccountResetPassword,
    /// Represents the account reverify variant for this public enum.
    AccountReverify,
    /// Represents the account verify variant for this public enum.
    AccountVerify(String),

    // Bot-related
    /// Represents the bot variant for this public enum.
    Bot(String),
    /// Represents the bot create variant for this public enum.
    BotCreate,
    /// Represents the bot invite variant for this public enum.
    BotInvite(String),
    /// Represents the bots owned variant for this public enum.
    BotsOwned,

    // Channel-related
    /// Represents the channel variant for this public enum.
    Channel(String),
    /// Represents the channel create variant for this public enum.
    ChannelCreate,
    /// Represents the channel invites variant for this public enum.
    ChannelInvites(String),
    /// Represents the channel join call variant for this public enum.
    ChannelJoinCall(String),
    /// Represents the channel end ring variant for this public enum.
    ChannelEndRing(String, String),
    /// Represents the channel members variant for this public enum.
    ChannelMembers(String),
    /// Represents the channel message variant for this public enum.
    ChannelMessage(String, String),
    /// Represents the channel message ack variant for this public enum.
    ChannelMessageAck(String, String),
    /// Represents the channel message bulk variant for this public enum.
    ChannelMessageBulk(String),
    /// Represents the channel message pin variant for this public enum.
    ChannelMessagePin(String, String),
    /// Represents the channel message reaction variant for this public enum.
    ChannelMessageReaction(String, String, String),
    /// Represents the channel message reactions variant for this public enum.
    ChannelMessageReactions(String, String),
    /// Represents the channel message search variant for this public enum.
    ChannelMessageSearch(String),
    /// Represents the channel messages variant for this public enum.
    ChannelMessages(String),
    /// Represents the channel permission variant for this public enum.
    ChannelPermission(String, String),
    /// Represents the channel permission default variant for this public enum.
    ChannelPermissionDefault(String),
    /// Represents the channel recipient variant for this public enum.
    ChannelRecipient(String, String),
    /// Represents the channel webhooks variant for this public enum.
    ChannelWebhooks(String),

    // Emoji-related
    /// Represents the emoji variant for this public enum.
    Emoji(String),

    // Invite-related
    /// Represents the invite variant for this public enum.
    Invite(String),

    // Relationship-related
    /// Represents the relationship block variant for this public enum.
    RelationshipBlock(String),
    /// Represents the relationship friend variant for this public enum.
    RelationshipFriend(String),
    /// Represents the relationship friends variant for this public enum.
    RelationshipFriends,
    /// Represents the relationship mutual variant for this public enum.
    RelationshipMutual(String),

    // Server-related
    /// Represents the server variant for this public enum.
    Server(String),
    /// Represents the server ack variant for this public enum.
    ServerAck(String),
    /// Represents the server ban variant for this public enum.
    ServerBan(String, String),
    /// Represents the server bans variant for this public enum.
    ServerBans(String),
    /// Represents the server channels variant for this public enum.
    ServerChannels(String),
    /// Represents the server create variant for this public enum.
    ServerCreate,
    /// Represents the server invites variant for this public enum.
    ServerInvites(String),
    /// Represents the server member variant for this public enum.
    ServerMember(String, String),
    /// Represents the server member experimental query variant for this public enum.
    ServerMemberExperimentalQuery(String),
    /// Represents the server members variant for this public enum.
    ServerMembers(String),
    /// Represents the server permission variant for this public enum.
    ServerPermission(String, String),
    /// Represents the server permission default variant for this public enum.
    ServerPermissionDefault(String),
    /// Represents the server role variant for this public enum.
    ServerRole(String, String),
    /// Represents the server roles variant for this public enum.
    ServerRoles(String),
    /// Represents the server role ranks variant for this public enum.
    ServerRoleRanks(String),

    // User-related
    /// Represents the user me variant for this public enum.
    UserMe,
    /// Represents the user variant for this public enum.
    User(String),
    /// Represents the user DM variant for this public enum.
    UserDM(String),
    /// Represents the user dms variant for this public enum.
    UserDMs,
    /// Represents the user default avatar variant for this public enum.
    UserDefaultAvatar(String),
    /// Represents the user flags variant for this public enum.
    UserFlags(String),
    /// Represents the user profile variant for this public enum.
    UserProfile(String),
    /// Represents the user safety variant for this public enum.
    UserSafety,
    /// Represents the user username variant for this public enum.
    UserUsername,

    // Instance-related
    /// Represents the instance config variant for this public enum.
    InstanceConfig,

    // Webhook-related
    /// Represents the webhook variant for this public enum.
    Webhook(String),
    /// Represents the webhook with token variant for this public enum.
    WebhookWithToken(String, String),
    /// Represents the webhook github variant for this public enum.
    WebhookGithub(String, String),

    // Onboarding-related
    /// Represents the onboarding hello variant for this public enum.
    OnboardingHello,
    /// Represents the onboarding complete variant for this public enum.
    OnboardingComplete,

    // MFA-related
    /// Represents the MFA variant for this public enum.
    Mfa,
    /// Represents the MFA methods variant for this public enum.
    MfaMethods,
    /// Represents the MFA recovery variant for this public enum.
    MfaRecovery,
    /// Represents the MFA ticket variant for this public enum.
    MfaTicket,
    /// Represents the MFA TOTP variant for this public enum.
    MfaTotp,

    // Sync-related
    /// Represents the sync settings fetch variant for this public enum.
    SyncSettingsFetch,
    /// Represents the sync settings set variant for this public enum.
    SyncSettingsSet,
    /// Represents the sync unreads variant for this public enum.
    SyncUnreads,

    // Push-related
    /// Represents the push subscribe variant for this public enum.
    PushSubscribe,
    /// Represents the push unsubscribe variant for this public enum.
    PushUnsubscribe,
}

impl Endpoint {
    /// Returns the path component of the endpoint URL.
    pub fn path(&self) -> String {
        match self {
            // Account-related
            Endpoint::Account => "/auth/account/".to_string(),
            Endpoint::AccountChangeEmail => "/auth/account/change/email".to_string(),
            Endpoint::AccountChangePassword => "/auth/account/change/password".to_string(),
            Endpoint::AccountCreate => "/auth/account/create".to_string(),
            Endpoint::AccountDelete => "/auth/account/delete".to_string(),
            Endpoint::AccountDisable => "/auth/account/disable".to_string(),
            Endpoint::AccountResetPassword => "/auth/account/reset_password".to_string(),
            Endpoint::AccountReverify => "/auth/account/reverify".to_string(),
            Endpoint::AccountVerify(code) => format!("/auth/account/verify/{}", code),

            // Bot-related
            Endpoint::Bot(bot_id) => format!("/bots/{}", bot_id),
            Endpoint::BotCreate => "/bots/create".to_string(),
            Endpoint::BotInvite(bot_id) => format!("/bots/{}/invite", bot_id),
            Endpoint::BotsOwned => "/bots/@me".to_string(),

            // Channel-related
            Endpoint::Channel(channel_id) => format!("/channels/{}", channel_id),
            Endpoint::ChannelCreate => "/channels/create".to_string(),
            Endpoint::ChannelInvites(channel_id) => format!("/channels/{}/invites", channel_id),
            Endpoint::ChannelJoinCall(channel_id) => format!("/channels/{}/join_call", channel_id),
            Endpoint::ChannelEndRing(channel_id, user_id) => {
                format!("/channels/{}/end_ring/{}", channel_id, user_id)
            }
            Endpoint::ChannelMembers(channel_id) => format!("/channels/{}/members", channel_id),
            Endpoint::ChannelMessage(channel_id, message_id) => {
                format!("/channels/{}/messages/{}", channel_id, message_id)
            }
            Endpoint::ChannelMessageAck(channel_id, message_id) => {
                format!("/channels/{}/ack/{}", channel_id, message_id)
            }
            Endpoint::ChannelMessageBulk(channel_id) => {
                format!("/channels/{}/messages/bulk", channel_id)
            }
            Endpoint::ChannelMessagePin(channel_id, message_id) => {
                format!("/channels/{}/messages/{}/pin", channel_id, message_id)
            }
            Endpoint::ChannelMessageReaction(channel_id, message_id, emoji_id) => {
                format!(
                    "/channels/{}/messages/{}/reactions/{}",
                    channel_id, message_id, emoji_id
                )
            }
            Endpoint::ChannelMessageReactions(channel_id, message_id) => {
                format!("/channels/{}/messages/{}/reactions", channel_id, message_id)
            }
            Endpoint::ChannelMessageSearch(channel_id) => {
                format!("/channels/{}/search", channel_id)
            }
            Endpoint::ChannelMessages(channel_id) => format!("/channels/{}/messages", channel_id),
            Endpoint::ChannelPermission(channel_id, role_id) => {
                format!("/channels/{}/permissions/{}", channel_id, role_id)
            }
            Endpoint::ChannelPermissionDefault(channel_id) => {
                format!("/channels/{}/permissions/default", channel_id)
            }
            Endpoint::ChannelRecipient(channel_id, recipient_id) => {
                format!("/channels/{}/recipients/{}", channel_id, recipient_id)
            }
            Endpoint::ChannelWebhooks(channel_id) => format!("/channels/{}/webhooks", channel_id),

            // Emoji-related
            Endpoint::Emoji(emoji_id) => format!("/custom/emoji/{}", emoji_id),

            // Invite-related
            Endpoint::Invite(invite_id) => format!("/invites/{}", invite_id),

            // Relationship-related
            Endpoint::RelationshipBlock(user_id) => format!("/users/{}/block", user_id),
            Endpoint::RelationshipFriend(user_id) => format!("/users/{}/friend", user_id),
            Endpoint::RelationshipFriends => "/users/friend".to_string(),
            Endpoint::RelationshipMutual(user_id) => format!("/users/{}/mutual", user_id),

            // Server-related
            Endpoint::Server(server_id) => format!("/servers/{}", server_id),
            Endpoint::ServerAck(server_id) => format!("/servers/{}/ack", server_id),
            Endpoint::ServerBan(server_id, member_id) => {
                format!("/servers/{}/bans/{}", server_id, member_id)
            }
            Endpoint::ServerBans(server_id) => format!("/servers/{}/bans", server_id),
            Endpoint::ServerChannels(server_id) => format!("/servers/{}/channels", server_id),
            Endpoint::ServerCreate => "/servers/create".to_string(),
            Endpoint::ServerInvites(server_id) => format!("/servers/{}/invites", server_id),
            Endpoint::ServerMember(server_id, member_id) => {
                format!("/servers/{}/members/{}", server_id, member_id)
            }
            Endpoint::ServerMemberExperimentalQuery(server_id) => {
                format!("/servers/{}/members_experimental_query", server_id)
            }
            Endpoint::ServerMembers(server_id) => format!("/servers/{}/members", server_id),
            Endpoint::ServerPermission(server_id, role_id) => {
                format!("/servers/{}/permissions/{}", server_id, role_id)
            }
            Endpoint::ServerPermissionDefault(server_id) => {
                format!("/servers/{}/permissions/default", server_id)
            }
            Endpoint::ServerRole(server_id, role_id) => {
                format!("/servers/{}/roles/{}", server_id, role_id)
            }
            Endpoint::ServerRoles(server_id) => format!("/servers/{}/roles", server_id),
            Endpoint::ServerRoleRanks(server_id) => format!("/servers/{}/roles/ranks", server_id),

            // User-related
            Endpoint::UserMe => "/users/@me".to_string(),
            Endpoint::User(user_id) => format!("/users/{}", user_id),
            Endpoint::UserDM(user_id) => format!("/users/{}/dm", user_id),
            Endpoint::UserDMs => "/users/dms".to_string(),
            Endpoint::UserDefaultAvatar(user_id) => format!("/users/{}/default_avatar", user_id),
            Endpoint::UserFlags(user_id) => format!("/users/{}/flags", user_id),
            Endpoint::UserProfile(user_id) => format!("/users/{}/profile", user_id),
            Endpoint::UserSafety => "/safety/report".to_string(),
            Endpoint::UserUsername => "/users/@me/username".to_string(),

            // Instance-related
            Endpoint::InstanceConfig => "/".to_string(),

            // Webhook-related
            Endpoint::Webhook(webhook_id) => format!("/webhooks/{}", webhook_id),
            Endpoint::WebhookWithToken(webhook_id, token) => {
                format!("/webhooks/{}/{}", webhook_id, token)
            }
            Endpoint::WebhookGithub(webhook_id, token) => {
                format!("/webhooks/{}/{}/github", webhook_id, token)
            }

            // Onboarding-related
            Endpoint::OnboardingHello => "/onboard/hello".to_string(),
            Endpoint::OnboardingComplete => "/onboard/complete".to_string(),

            // MFA-related
            Endpoint::Mfa => "/auth/mfa/".to_string(),
            Endpoint::MfaMethods => "/auth/mfa/methods".to_string(),
            Endpoint::MfaRecovery => "/auth/mfa/recovery".to_string(),
            Endpoint::MfaTicket => "/auth/mfa/ticket".to_string(),
            Endpoint::MfaTotp => "/auth/mfa/totp".to_string(),

            // Sync-related
            Endpoint::SyncSettingsFetch => "/sync/settings/fetch".to_string(),
            Endpoint::SyncSettingsSet => "/sync/settings/set".to_string(),
            Endpoint::SyncUnreads => "/sync/unreads".to_string(),

            // Push-related
            Endpoint::PushSubscribe => "/push/subscribe".to_string(),
            Endpoint::PushUnsubscribe => "/push/unsubscribe".to_string(),
        }
    }
}
