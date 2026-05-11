/// Re-exports all public Stoat API model types from their modules.
pub use {
    account::*, attachment::*, bot::*, channel::*, embed::*, emoji::*, event::*, invite::*,
    member::*, message::*, mfa::*, misc::*, permission::*, server::*, sync::*, user::*, webhook::*,
};

mod account;
mod attachment;
mod bot;
mod channel;
mod embed;
mod emoji;
mod event;
mod invite;
mod member;
mod message;
mod mfa;
mod misc;
mod permission;
mod server;
mod sync;
mod user;
mod webhook;

/// Convenience type alias for ID values used throughout the Kaho crate.
pub type Id = String;
