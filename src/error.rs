use {
    reqwest::{Error as ReqwestError, Response},
    serde::Deserialize,
    std::result::Result as StdResult,
    thiserror::Error,
    tokio_tungstenite::tungstenite::Error as WebSocketError,
};

pub type KahoResult<T = (), E = KahoError> = StdResult<T, E>;

/// Top-level error type for all operations within the `kaho` crate.
#[derive(Debug, Error)]
pub enum KahoError {
    /// Network or HTTP-related error via `reqwest`.
    #[error("HTTP error: {0}")]
    Http(#[from] ReqwestError),

    /// Received a response with a non-success status code.
    #[error("Request failed with non-success status: {0:?}")]
    FailedRequest(Response),

    /// WebSocket-level error.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WebSocketError),

    /// Error encountered during authentication with the Stoat API.
    #[error("Authentication failure: {0}")]
    Auth(#[from] AuthError),

    /// Any other unknown or uncategorized error.
    #[error("Unhandled error: {0}")]
    Other(String),
}

/// Authentication-specific errors encountered during login or token validation.
#[derive(Debug, Error, Deserialize, Clone, Copy, PartialEq)]
pub enum AuthError {
    /// Generic fallback error.
    #[error("Uncategorized authentication error")]
    LabelMe,

    /// Internal issue on the server side.
    #[error("Server encountered an internal error")]
    InternalError,

    /// Provided token is invalid or expired.
    #[error("Invalid session token")]
    InvalidSession,

    #[error("Invalid username")]
    OnboardingNotFinished,

    /// Attempted to authenticate while already authenticated.
    #[error("Session already active")]
    AlreadyAuthenticated,
}
