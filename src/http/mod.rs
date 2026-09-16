/// Re-exports the public HTTP client and HTTP configuration types.
pub use {
    client::*,
    config::*,
    rate_limit::{RateLimitedResponse, RateLimiter},
};

mod client;
mod config;
/// Endpoint variants used to build typed Stoat REST API paths.
pub mod endpoint;
mod rate_limit;
