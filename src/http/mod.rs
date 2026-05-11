/// Re-exports the public HTTP client and HTTP configuration types.
pub use {
    client::*,
    config::*,
};

mod client;
mod config;
/// Endpoint variants used to build typed Stoat REST API paths.
pub mod endpoint;
