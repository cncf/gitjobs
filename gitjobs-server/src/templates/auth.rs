//! This module defines some templates used for authentication.

use axum_messages::Message;
use rinja::Template;
use serde::{Deserialize, Serialize};

/// Log in page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/log_in.html")]
pub(crate) struct LogInPage {
    pub logged_in: bool,
    pub messages: Vec<Message>,

    pub next_url: Option<String>,
}

/// Sign up page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "auth/sign_up.html")]
pub(crate) struct SignUpPage {
    pub logged_in: bool,

    pub next_url: Option<String>,
}
