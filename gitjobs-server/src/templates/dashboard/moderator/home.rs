//! This module defines some templates and types used in the moderator
//! dashboard home page.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};

use crate::templates::{Config, PageId, auth::User, dashboard::moderator::jobs, filters};

// Pages templates.

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/moderator/home.html")]
pub(crate) struct Page {
    pub cfg: Config,
    pub content: Content,
    pub page_id: PageId,
    pub messages: Vec<Message>,
    pub user: User,
}

// Types.

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    LiveJobs(jobs::LivePage),
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

/// Tab selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Tab {
    LiveJobs,
    #[default]
    PendingJobs,
}
