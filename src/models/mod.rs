pub use {
    account::*,
    attachment::*,
    bot::*,
    channel::*,
    embed::*,
    emoji::*,
    event::*,
    member::*,
    message::*,
    invite::*,
    permission::*,
    server::*,
    user::*,
};

mod account;
mod attachment;
mod bot;
mod channel;
mod embed;
mod emoji;
mod event;
mod member;
mod message;
mod invite;
mod permission;
mod server;
mod user;

/// Unique Stoat object identifier.
pub type Id = String;
