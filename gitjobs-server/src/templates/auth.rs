//! This module defines some templates used for authentication.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};

use crate::{
    auth::UserSummary,
    templates::{PageId, filters},
};

// Pages templates.

/// Log in page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/log_in.html")]
pub(crate) struct LogInPage {
    pub page_id: PageId,
    pub logged_in: bool,
    pub messages: Vec<Message>,

    pub name: Option<String>,
    pub next_url: Option<String>,
    pub username: Option<String>,
}

/// Sign up page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/sign_up.html")]
pub(crate) struct SignUpPage {
    pub page_id: PageId,
    pub logged_in: bool,
    pub messages: Vec<Message>,

    pub name: Option<String>,
    pub next_url: Option<String>,
    pub username: Option<String>,
}

/// Update user page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/update_user.html")]
pub(crate) struct UpdateUserPage {
    pub user_summary: UserSummary,
}
