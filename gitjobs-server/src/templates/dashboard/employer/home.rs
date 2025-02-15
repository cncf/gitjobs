//! This module defines some templates and types used in the employer dashboard
//! home page.

use rinja::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{
    auth,
    dashboard::employer,
    filters,
    helpers::{build_image_url, find_employer},
};

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/employer/home.html")]
pub(crate) struct Page {
    pub content: Content,
    pub employers: Vec<employer::employers::EmployerSummary>,
    pub logged_in: bool,

    pub name: Option<String>,
    pub selected_employer_id: Option<Uuid>,
    pub username: Option<String>,
}

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    Account(auth::UpdateUserPage),
    EmployerInitialSetup(employer::employers::InitialSetupPage),
    Jobs(employer::jobs::ListPage),
    Profile(employer::employers::UpdatePage),
}

impl Content {
    /// Check if the content is the account page.
    fn is_account(&self) -> bool {
        matches!(self, Content::Account(_))
    }

    /// Check if the content is the employer initial setup page.
    fn is_employer_initial_setup(&self) -> bool {
        matches!(self, Content::EmployerInitialSetup(_))
    }

    /// Check if the content is the jobs page.
    fn is_jobs(&self) -> bool {
        matches!(self, Content::Jobs(_))
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
            Content::EmployerInitialSetup(template) => write!(f, "{}", template.render()?),
            Content::Jobs(template) => write!(f, "{}", template.render()?),
            Content::Profile(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) enum Tab {
    Account,
    EmployerInitialSetup,
    #[default]
    Jobs,
    Profile,
}

impl From<Option<&String>> for Tab {
    fn from(tab: Option<&String>) -> Self {
        match tab.map(String::as_str) {
            Some("account") => Tab::Account,
            Some("employer-initial-setup") => Tab::EmployerInitialSetup,
            Some("profile") => Tab::Profile,
            _ => Tab::Jobs,
        }
    }
}
