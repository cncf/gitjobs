//! This module defines some templates and types used in the job seeker
//! dashboard home page.

use rinja::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{auth, dashboard::job_seeker, filters, CurrentPage};

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/job_seeker/home.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct Page {
    pub content: Content,
    #[allow(dead_code)]
    pub current_page: CurrentPage,
    pub logged_in: bool,

    pub name: Option<String>,
    pub username: Option<String>,
}

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Content {
    Account(auth::UpdateUserPage),
    Profile(job_seeker::profile::UpdatePage),
}

impl Content {
    /// Check if the content is the account page.
    fn is_account(&self) -> bool {
        matches!(self, Content::Account(_))
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
            Content::Profile(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) enum Tab {
    Account,
    #[default]
    Profile,
}

impl From<Option<&String>> for Tab {
    fn from(tab: Option<&String>) -> Self {
        match tab.map(String::as_str) {
            Some("account") => Tab::Account,
            _ => Tab::Profile,
        }
    }
}
