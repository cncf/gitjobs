//! This module defines some templates and types used in the job seeker
//! dashboard home page.

use rinja::Template;
use serde::{Deserialize, Serialize};

use crate::templates::dashboard::job_seeker;

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/job_seeker/home.html")]
pub(crate) struct Page {
    pub content: Content,
}

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    Profile(job_seeker::profile::UpdatePage),
}

impl Content {
    /// Check if the content is the profile page.
    fn is_profile(&self) -> bool {
        matches!(self, Content::Profile(_))
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Profile(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) enum Tab {
    #[default]
    Profile,
}

impl From<Option<&String>> for Tab {
    fn from(_tab: Option<&String>) -> Self {
        Tab::Profile
    }
}
