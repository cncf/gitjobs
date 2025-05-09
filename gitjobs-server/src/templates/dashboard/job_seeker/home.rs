//! This module defines some templates and types used in the job seeker
//! dashboard home page.

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

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/job_seeker/home.html")]
pub(crate) struct Page {
    pub cfg: Config,
    pub content: Content,
    pub page_id: PageId,
    pub messages: Vec<Message>,
    pub user: User,

    pub auth_provider: Option<String>,
}

// Types.

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Content {
    Account(auth::UpdateUserPage),
    Applications(job_seeker::applications::ApplicationsPage),
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

/// Tab selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Tab {
    Account,
    Applications,
    #[default]
    Profile,
}
