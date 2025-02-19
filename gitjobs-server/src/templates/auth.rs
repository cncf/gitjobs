//! This module defines some templates used for authentication.

use axum_messages::Message;
use rinja::Template;
use serde::{Deserialize, Serialize};

use crate::{
    auth::UserSummary,
    templates::{filters, CurrentPage},
};

/// Log in page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/log_in.html")]
pub(crate) struct LogInPage {
    pub current_page: CurrentPage,
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
    pub current_page: CurrentPage,
    pub logged_in: bool,

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
