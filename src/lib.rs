/// Re-exports the crate-wide `KahoResult` alias for convenient public use.
#[doc(hidden)]
pub use error::KahoResult;
/// Optional in-memory cache for users, servers, roles, members, channels, and messages.

#[cfg(feature = "cache")]
pub mod cache;
/// High-level client types used to connect HTTP and gateway functionality.
pub mod client;
/// Error types and result aliases returned by Kaho operations.
pub mod error;
/// Gateway WebSocket client, configuration, and event streaming utilities.
pub mod gateway;
/// HTTP client, endpoint routing, and REST configuration utilities.
pub mod http;
/// Public data models used by the Stoat API and gateway events.
pub mod models;
/// Optional type store utilities for attaching user-defined state to the client.

#[cfg(feature = "type-store")]
pub mod type_store;
