//! Templates for notification-related emails and messages.

use askama::Template;
use serde::{Deserialize, Serialize};

// Emails templates.

/// Template for email verification notification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/email_verification.html")]
pub(crate) struct EmailVerification {
    /// Verification link for the user to confirm their email address.
    pub link: String,
}

/// Template for team invitation notification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/team_invitation.html")]
pub(crate) struct TeamInvitation {
    /// Link to invitations page.
    pub link: String,
}
