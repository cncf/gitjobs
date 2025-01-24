//! This module defines some templates and types used in the home page.

use rinja::Template;
use serde::{Deserialize, Serialize};

use super::{employers, jobs};

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/home.html")]
pub(crate) struct Page {
    pub content: Content,
}

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    Jobs(jobs::ListPage),
    Settings(employers::UpdatePage),
}

impl Content {
    /// Check if the content is the jobs page.
    fn is_jobs(&self) -> bool {
        matches!(self, Content::Jobs(_))
    }

    /// Check if the content is the settings page.
    fn is_settings(&self) -> bool {
        matches!(self, Content::Settings(_))
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Jobs(template) => write!(f, "{}", template.render()?),
            Content::Settings(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) enum Tab {
    #[default]
    Jobs,
    Settings,
}

impl From<Option<&String>> for Tab {
    fn from(tab: Option<&String>) -> Self {
        match tab.map(String::as_str) {
            Some("settings") => Tab::Settings,
            _ => Tab::Jobs,
        }
    }
}
