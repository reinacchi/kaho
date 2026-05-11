/// Re-exports the crate-wide `KahoResult` alias for convenient public use.
#[doc(hidden)]
pub use error::KahoResult;
/// Error types and result aliases returned by Kaho operations.
pub mod error;
/// High-level client types used to connect HTTP and gateway functionality.
pub mod client;
/// Gateway WebSocket client, configuration, and event streaming utilities.
pub mod gateway;
/// HTTP client, endpoint routing, and REST configuration utilities.
pub mod http;
/// Public data models used by the Stoat API and gateway events.

pub mod models;
/// Optional in-memory cache for users, servers, channels, and messages.

#[cfg(feature = "cache")]
pub mod cache;
/// Optional type store utilities for attaching user-defined state to the client.

#[cfg(feature = "type-store")]
pub mod type_store;
