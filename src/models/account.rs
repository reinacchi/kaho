use serde::{Deserialize, Serialize};

use crate::{http::HttpClient, KahoResult};

/// Account details returned by the account endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Account {
    /// The unique ID assigned to the `Account` by the Stoat API.
    #[serde(rename = "_id")]
    pub id: String,
    /// The email address associated with the account operation.
    pub email: String,
}

impl Account {
    /// Fetch the `Account`.
    pub async fn fetch(http: &HttpClient) -> KahoResult<Self> {
        http.fetch_account().await
    }

    /// Change the account password.
    pub async fn change_password(
        http: &HttpClient,
        payload: impl Into<AccountChangePassword>,
    ) -> KahoResult {
        http.change_password(payload).await
    }

    /// Change the account email address.
    pub async fn change_email(
        http: &HttpClient,
        payload: impl Into<AccountChangeEmail>,
    ) -> KahoResult {
        http.change_email(payload).await
    }

    /// Request account deletion.
    pub async fn request_deletion(
        http: &HttpClient,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        http.delete_account(payload).await
    }

    /// Confirm deletion.
    pub async fn confirm_deletion(
        http: &HttpClient,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        http.confirm_account_deletion(payload).await
    }

    /// Disable the `Account`.
    pub async fn disable(
        http: &HttpClient,
        payload: impl Into<AccountPasswordConfirmation>,
    ) -> KahoResult {
        http.disable_account(payload).await
    }
}

/// Represents an account create value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountCreate {
    /// The email address associated with the account operation.
    pub email: String,
    /// The password value supplied for account authentication or confirmation.
    pub password: String,
    /// The invite code or invite identifier used by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite: Option<String>,
    /// The captcha response used to satisfy verification requirements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha: Option<String>,
}

/// Represents an account resend verification value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountResendVerification {
    /// The email address associated with the account operation.
    pub email: String,
    /// The captcha response used to satisfy verification requirements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha: Option<String>,
}

/// Represents an account password confirmation value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountPasswordConfirmation {
    /// The password value supplied for account authentication or confirmation.
    pub password: String,
}

/// Represents an account change password value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountChangePassword {
    /// The password value supplied for account authentication or confirmation.
    pub password: String,
    /// The replacement password to set on the account.
    pub new_password: String,
}

/// Represents an account change email value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountChangeEmail {
    /// The email address associated with the account operation.
    pub email: String,
    /// The password value supplied for account authentication or confirmation.
    pub password: String,
}

/// Represents an account send password reset value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountSendPasswordReset {
    /// The email address associated with the account operation.
    pub email: String,
    /// The captcha response used to satisfy verification requirements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captcha: Option<String>,
}

/// Represents an account password reset value used by the Stoat API models and endpoints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountPasswordReset {
    /// The token used to authenticate or execute this API resource.
    pub token: String,
    /// The password value supplied for account authentication or confirmation.
    pub password: String,
}
