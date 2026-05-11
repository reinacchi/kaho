use {
    reqwest::{header::InvalidHeaderValue, Error as ReqwestError, Response},
    serde::Deserialize,
    std::result::Result as StdResult,
    thiserror::Error,
    tokio_tungstenite::tungstenite::Error as WebSocketError,
};

/// Convenience result type used by Kaho operations.
pub type KahoResult<T = (), E = KahoError> = StdResult<T, E>;

/// Top-level error type for all operations within the `kaho` crate.
#[derive(Debug, Error)]
pub enum KahoError {
    /// Represents the http variant for this public enum.
    #[error("HTTP error: {0}")]
    Http(#[from] ReqwestError),

    /// Header value could not be represented as a valid HTTP header.
    #[error("Invalid HTTP header value: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),

    /// Received a response with a non-success status code.
    #[error("Request failed with non-success status: {0:?}")]
    FailedRequest(Response),

    /// Represents the web socket variant for this public enum.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WebSocketError),

    /// Error encountered during authentication with the Stoat API.
    #[error("Authentication failure: {0}")]
    Auth(#[from] AuthError),

    /// Represents the other variant for this public enum.
    #[error("Unhandled error: {0}")]
    Other(String),
}

/// Authentication-specific errors encountered during login or token validation.
#[derive(Debug, Error, Deserialize, Clone, Copy, PartialEq)]
pub enum AuthError {
    /// Represents the label me variant for this public enum.
    #[error("Uncategorized authentication error")]
    LabelMe,

    /// Internal issue on the server side.
    #[error("Server encountered an internal error")]
    InternalError,

    /// Represents the invalid session variant for this public enum.
    #[error("Invalid session token")]
    InvalidSession,

    /// The account has not completed onboarding.
    #[error("Invalid username")]
    OnboardingNotFinished,

    /// Represents the already authenticated variant for this public enum.
    #[error("Session already active")]
    AlreadyAuthenticated,
}
