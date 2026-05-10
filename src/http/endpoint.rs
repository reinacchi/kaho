#[derive(Debug, Clone)]
pub enum Endpoint {
    // Bot-related
    Bot(String),
    BotCreate(),
    BotInvite(String),

    // Channel-related
    Channel(String),
    ChannelCreate(),
    ChannelInvites(String),
    ChannelJoinCall(String),
    ChannelMembers(String),
    ChannelMessage(String, String),
    ChannelMessageAck(String, String),
    ChannelMessageBulk(String),
    ChannelMessagePin(String, String),
    ChannelMessageReaction(String, String, String),
    ChannelMessageReactions(String, String),
    ChannelMessageSearch(String),
    ChannelMessages(String),
    ChannelPermission(String, String),
    ChannelRecipient(String, String),
    ChannelWebhooks(String),

    // Emoji-related
    Emoji(String),

    // Invite-related
    Invite(String),

    // Relationship-related
    RelationshipBlock(String),
    RelationshipFriend(String),
    RelationshipFriends(),
    RelationshipMutual(String),

    // Server-related
    Server(String),
    ServerAck(String),
    ServerBan(String, String),
    ServerBans(String),
    ServerChannels(String),
    ServerCreate(),
    ServerInvites(String),
    ServerMember(String, String),
    ServerMemberExperimentalQuery(String),
    ServerMembers(String),
    ServerPermission(String, String),
    ServerRole(String, String),
    ServerRoles(String),

    // User-related
    User(String),
    UserDM(String),
    UserDMs(),
    UserDefaultAvatar(String),
    UserFlags(String),
    UserProfile(String),
    UserSafety(),
    UserUsername(String),
}

impl Endpoint {
    /// Returns the path component of the endpoint URL
    pub fn path(&self) -> String {
        match self {
            // Bot-related
            Endpoint::Bot(bot_id) => format!("/bots/{}", bot_id),
            Endpoint::BotCreate() => format!("/bots/create"),
            Endpoint::BotInvite(bot_id) => format!("/bots/{}/invite", bot_id),

            // Channel-related
            Endpoint::Channel(channel_id) => format!("/channels/{}", channel_id),
            Endpoint::ChannelCreate() => format!("/channels/create"),
            Endpoint::ChannelInvites(channel_id) => format!("/channels/{}/invites", channel_id),
            Endpoint::ChannelJoinCall(channel_id) => format!("/channels/{}/join_call", channel_id),
            Endpoint::ChannelMembers(channel_id) => {
                format!("/channels/{}/members", channel_id)
            }
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
                format!("/channels/{}/messages/{}/reactions/{}", channel_id, message_id, emoji_id)
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
            Endpoint::RelationshipFriends() => format!("/users/friend"),
            Endpoint::RelationshipMutual(user_id) => format!("/users/{}/mutual", user_id),

            // Server-related
            Endpoint::Server(server_id) => format!("/servers/{}", server_id),
            Endpoint::ServerAck(server_id) => format!("/servers/{}/ack", server_id),
            Endpoint::ServerBan(server_id, member_id) => {
                format!("/servers/{}/bans/{}", server_id, member_id)
            }
            Endpoint::ServerBans(server_id) => format!("/servers/{}/bans", server_id),
            Endpoint::ServerChannels(server_id) => format!("/servers/{}/channels", server_id),
            Endpoint::ServerCreate() => format!("/servers/create"),
            Endpoint::ServerInvites(server_id) => format!("/servers/{}/invites", server_id),
            Endpoint::ServerMember(server_id, member_id) => {
                format!("/servers/{}/members/{}", server_id, member_id)
            }
            Endpoint::ServerMemberExperimentalQuery(server_id) => format!("/servers/{}/members_experimental_query", server_id),
            Endpoint::ServerMembers(server_id) => format!("/servers/{}/members", server_id),
            Endpoint::ServerPermission(server_id, role_id) => {
                format!("/server/{}/permissions/{}", server_id, role_id)
            }
            Endpoint::ServerRole(server_id, role_id) => {
                format!("/servers/{}/roles/{}", server_id, role_id)
            }
            Endpoint::ServerRoles(server_id) => format!("/server/{}/roles", server_id),

            // User-related
            Endpoint::User(user_id) => format!("/users/{}", user_id),
            Endpoint::UserDM(user_id) => format!("/users/{}/dm", user_id),
            Endpoint::UserDMs() => format!("/users/dms"),
            Endpoint::UserDefaultAvatar(user_id) => format!("/users/{}/default_avatar", user_id),
            Endpoint::UserFlags(user_id) => format!("/users/{}/flags", user_id),
            Endpoint::UserProfile(user_id) => format!("/users/{}/profile", user_id),
            Endpoint::UserSafety() => format!("/safety/report"),
            Endpoint::UserUsername(user_id) => format!("/users/{}/username", user_id),
        }
    }
}
