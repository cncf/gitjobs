//! This module defines some templates used in notifications.

use rinja::Template;
use serde::{Deserialize, Serialize};

/// Email verification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/email_verification.html")]
pub(crate) struct EmailVerification {
    pub link: String,
}
