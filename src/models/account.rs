use serde::{Deserialize, Serialize};

use crate::{http::HttpClient, KahoResult};

/// Account details returned by the account endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Account {
    /// Account ID.
    #[serde(rename = "_id")]
    pub id: String,
    /// Account email.
    pub email: String,
}

impl Account {
    /// Fetch the current account again.
    pub async fn fetch(http: &HttpClient) -> KahoResult<Self> {
        http.fetch_account().await
    }

    /// Change this account's password.
    pub async fn change_password(http: &HttpClient, payload: impl Into<AccountChangePassword>) -> KahoResult {
        http.change_password(payload).await
    }

    /// Change this account's email.
    pub async fn change_email(http: &HttpClient, payload: impl Into<AccountChangeEmail>) -> KahoResult {
        http.change_email(payload).await
    }

    /// Request deletion for this account.
    pub async fn request_deletion(http: &HttpClient, payload: impl Into<AccountPasswordConfirmation>) -> KahoResult {
        http.delete_account(payload).await
    }

    /// Confirm deletion for this account.
    pub async fn confirm_deletion(http: &HttpClient, payload: impl Into<AccountPasswordConfirmation>) -> KahoResult {
        http.confirm_account_deletion(payload).await
    }

    /// Disable this account.
    pub async fn disable(http: &HttpClient, payload: impl Into<AccountPasswordConfirmation>) -> KahoResult {
        http.disable_account(payload).await
    }
}

/// Payload for creating an account.
#[derive(Clone, Debug, Serialize)]
pub struct AccountCreate {
    /// Email address.
    pub email: String,
    /// Password.
    pub password: String,
    /// Invite code, when required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite: Option<String>,
    /// Captcha verification code, when required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha: Option<String>,
}

/// Payload for resending account verification.
#[derive(Clone, Debug, Serialize)]
pub struct AccountResendVerification {
    /// Email address.
    pub email: String,
    /// Captcha verification code, when required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha: Option<String>,
}

/// Payload for account actions confirmed by password.
#[derive(Clone, Debug, Serialize)]
pub struct AccountPasswordConfirmation {
    /// Current password.
    pub password: String,
}

/// Payload for changing an account password.
#[derive(Clone, Debug, Serialize)]
pub struct AccountChangePassword {
    /// Current password.
    pub password: String,
    /// New password.
    pub new_password: String,
}

/// Payload for changing an account email.
#[derive(Clone, Debug, Serialize)]
pub struct AccountChangeEmail {
    /// New email address.
    pub email: String,
    /// Current password.
    pub password: String,
}

/// Payload for requesting a password reset email.
#[derive(Clone, Debug, Serialize)]
pub struct AccountSendPasswordReset {
    /// Email address.
    pub email: String,
    /// Captcha verification code, when required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha: Option<String>,
}

/// Payload for completing a password reset.
#[derive(Clone, Debug, Serialize)]
pub struct AccountPasswordReset {
    /// Reset token.
    pub token: String,
    /// New password.
    pub password: String,
}
