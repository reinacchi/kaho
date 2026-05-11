/// Re-exports all public Stoat API model types from their modules.
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
    webhook::*,
    sync::*,
    misc::*,
    mfa::*,
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
mod webhook;
mod sync;
mod misc;
mod mfa;
mod invite;
mod permission;
mod server;
mod user;

/// Convenience type alias for ID values used throughout the Kaho crate.
pub type Id = String;
