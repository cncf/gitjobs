//! Templates and types for authentication-related pages and user info.

use askama::Template;
use axum_messages::{Level, Message};
use garde::Validate;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{AuthSession, UserSummary},
    config::LoginOptions,
    templates::{Config, PageId, filters},
    validation::{MAX_LEN_DISPLAY_NAME, MAX_LEN_M, MAX_LEN_S, MIN_PASSWORD_LEN, trimmed_non_empty},
};

// Pages templates.

/// Template for the log in page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/log_in.html")]
pub(crate) struct LogInPage {
    /// Server configuration.
    pub cfg: Config,
    /// Login options.
    pub login: LoginOptions,
    /// Identifier for the current page.
    pub page_id: PageId,
    /// Flash or status messages to display.
    pub messages: Vec<Message>,
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
    /// Next URL to redirect to after login, if any.
    pub next_url: Option<String>,
}

/// Template for the sign up page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/sign_up.html")]
pub(crate) struct SignUpPage {
    /// Server configuration.
    pub cfg: Config,
    /// Login options.
    pub login: LoginOptions,
    /// Identifier for the current page.
    pub page_id: PageId,
    /// Flash or status messages to display.
    pub messages: Vec<Message>,
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
    /// Next URL to redirect to after sign up, if any.
    pub next_url: Option<String>,
}

/// Template for the update user page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/update_user.html")]
pub(crate) struct UpdateUserPage {
    /// User summary information for updating the user.
    pub user_summary: UserSummary,
}

// Types.

/// Input for updating user details.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct UpdateUserDetailsInput {
    /// User's display name.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DISPLAY_NAME))]
    pub name: String,
}

/// User information for authentication templates and session state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct User {
    /// Whether the user has a profile.
    pub has_profile: bool,
    /// Whether the user is logged in.
    pub logged_in: bool,
    /// Whether the user is a moderator.
    pub moderator: bool,

    /// Display name of the user, if any.
    pub name: Option<String>,
    /// Username, if any.
    pub username: Option<String>,
}

/// Conversion from `AuthSession` to User for template rendering.
impl From<AuthSession> for User {
    fn from(session: AuthSession) -> Self {
        let user = session.user.as_ref();

        Self {
            has_profile: user.is_some_and(|u| u.has_profile),
            logged_in: user.is_some(),
            moderator: user.is_some_and(|u| u.moderator),
            name: user.map(|u| u.name.clone()),
            username: user.map(|u| u.username.clone()),
        }
    }
}

/// Input for updating a user's password.
#[derive(Clone, Serialize, Deserialize, Validate)]
pub(crate) struct UserPassword {
    /// The new password to set.
    #[garde(length(min = MIN_PASSWORD_LEN, max = MAX_LEN_S))]
    pub new_password: String,
    /// The user's current password.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_M))]
    pub old_password: String,
}
