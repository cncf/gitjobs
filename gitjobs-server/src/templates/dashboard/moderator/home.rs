//! Templates and types for the moderator dashboard home page.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};

use crate::templates::{Config, PageId, auth::User, dashboard::moderator::jobs, filters};

// Pages templates.

/// Template for the moderator dashboard home page.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/moderator/home.html")]
pub(crate) struct Page {
    /// Server configuration.
    pub cfg: Config,
    /// Content section for the dashboard.
    pub content: Content,
    /// Identifier for the current page.
    pub page_id: PageId,
    /// Flash or status messages to display.
    pub messages: Vec<Message>,
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
}

// Types.

/// Content section for the moderator dashboard home page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    /// Live jobs page content.
    LiveJobs(jobs::LivePage),
    /// Pending jobs page content.
    PendingJobs(jobs::PendingPage),
}

impl Content {
    /// Check if the content is the live jobs page.
    fn is_live_jobs(&self) -> bool {
        matches!(self, Content::LiveJobs(_))
    }

    /// Check if the content is the pending jobs page.
    fn is_pending_jobs(&self) -> bool {
        matches!(self, Content::PendingJobs(_))
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::LiveJobs(template) => write!(f, "{}", template.render()?),
            Content::PendingJobs(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selection for the moderator dashboard home page.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Tab {
    /// Live jobs tab.
    LiveJobs,
    /// Pending jobs tab (default).
    #[default]
    PendingJobs,
}
