//! This module defines some templates and types used in the employer dashboard
//! home page.

use rinja::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{dashboard::employer, filters};

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/employer/home.html")]
pub(crate) struct Page {
    pub content: Content,
    pub employers: Vec<employer::employers::EmployerSummary>,

    pub selected_employer_id: Option<Uuid>,
}

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    EmployerInitialSetup(employer::employers::InitialSetupPage),
    Jobs(employer::jobs::ListPage),
    Settings(employer::employers::UpdatePage),
}

impl Content {
    /// Check if the content is the employer initial setup page.
    fn is_employer_initial_setup(&self) -> bool {
        matches!(self, Content::EmployerInitialSetup(_))
    }

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
            Content::EmployerInitialSetup(template) => write!(f, "{}", template.render()?),
            Content::Jobs(template) => write!(f, "{}", template.render()?),
            Content::Settings(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) enum Tab {
    EmployerInitialSetup,
    #[default]
    Jobs,
    Settings,
}

impl From<Option<&String>> for Tab {
    fn from(tab: Option<&String>) -> Self {
        match tab.map(String::as_str) {
            Some("employer-initial-setup") => Tab::EmployerInitialSetup,
            Some("settings") => Tab::Settings,
            _ => Tab::Jobs,
        }
    }
}
