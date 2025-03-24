//! This module defines some templates used for authentication.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{AuthSession, UserSummary},
    templates::{PageId, filters},
};

// Pages templates.

/// Log in page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/log_in.html")]
pub(crate) struct LogInPage {
    pub page_id: PageId,
    pub messages: Vec<Message>,
    pub user: User,

    pub next_url: Option<String>,
}

/// Sign up page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/sign_up.html")]
pub(crate) struct SignUpPage {
    pub page_id: PageId,
    pub messages: Vec<Message>,
    pub user: User,

    pub next_url: Option<String>,
}

/// Update user page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/update_user.html")]
pub(crate) struct UpdateUserPage {
    pub user_summary: UserSummary,
}

// Types.

/// User information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct User {
    pub has_profile: bool,
    pub logged_in: bool,

    pub name: Option<String>,
    pub username: Option<String>,
}

impl From<AuthSession> for User {
    fn from(session: AuthSession) -> Self {
        let user = session.user.as_ref();

        Self {
            has_profile: user.is_some_and(|u| u.has_profile),
            logged_in: user.is_some(),
            name: user.map(|u| u.name.clone()),
            username: user.map(|u| u.username.clone()),
        }
    }
}
