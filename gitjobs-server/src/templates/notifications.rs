//! This module defines some templates used in notifications.

use askama::Template;
use serde::{Deserialize, Serialize};

// Emails templates.

/// Email verification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/email_verification.html")]
pub(crate) struct EmailVerification {
    pub link: String,
}
