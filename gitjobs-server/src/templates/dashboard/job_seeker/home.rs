//! Templates and types for the job seeker dashboard home page.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};

use crate::templates::{
    Config, PageId,
    auth::{self, User},
    dashboard::job_seeker,
    filters,
};

// Pages templates.

/// Home page template for the job seeker dashboard.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/job_seeker/home.html")]
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

/// Content section for the job seeker dashboard home page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Content {
    /// Account update page content.
    Account(auth::UpdateUserPage),
    /// Applications list page content.
    Applications(job_seeker::applications::ApplicationsPage),
    /// Profile update page content.
    Profile(job_seeker::profile::UpdatePage),
}

impl Content {
    /// Check if the content is the account page.
    fn is_account(&self) -> bool {
        matches!(self, Content::Account(_))
    }

    /// Check if the content is the applications page.
    #[allow(dead_code)]
    fn is_applications(&self) -> bool {
        matches!(self, Content::Applications(_))
    }

    /// Check if the content is the profile page.
    fn is_profile(&self) -> bool {
        matches!(self, Content::Profile(_))
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Account(template) => write!(f, "{}", template.render()?),
            Content::Applications(template) => write!(f, "{}", template.render()?),
            Content::Profile(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selection for the job seeker dashboard home page.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Tab {
    /// Account tab.
    Account,
    /// Applications tab.
    Applications,
    /// Profile tab (default).
    #[default]
    Profile,
}
