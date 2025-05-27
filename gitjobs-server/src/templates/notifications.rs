//! Templates for notification-related emails and messages.

use askama::Template;
use serde::{Deserialize, Serialize};

use super::jobboard::jobs::Job;

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

// Slack templates.

/// Template for the new job published Slack notification.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "notifications/slack_job_published.md")]
pub(crate) struct JobPublished {
    /// Job details.
    pub job: Job,
}
