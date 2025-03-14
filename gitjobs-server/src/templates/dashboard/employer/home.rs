//! This module defines some templates and types used in the employer dashboard
//! home page.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{
    PageId, auth,
    dashboard::employer,
    filters,
    helpers::{build_dashboard_image_url, find_employer},
};

// Pages templates.

/// Home page template.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/employer/home.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct Page {
    pub content: Content,
    pub employers: Vec<employer::employers::EmployerSummary>,
    pub logged_in: bool,
    pub messages: Vec<Message>,
    pub page_id: PageId,

    pub name: Option<String>,
    pub selected_employer_id: Option<Uuid>,
    pub username: Option<String>,
}

// Types.

/// Content section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    Account(auth::UpdateUserPage),
    Applications(employer::applications::ApplicationsPage),
    EmployerInitialSetup(employer::employers::InitialSetupPage),
    Jobs(employer::jobs::ListPage),
    Profile(employer::employers::UpdatePage),
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
            Content::Applications(template) => write!(f, "{}", template.render()?),
            Content::EmployerInitialSetup(template) => write!(f, "{}", template.render()?),
            Content::Jobs(template) => write!(f, "{}", template.render()?),
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
    EmployerInitialSetup,
    #[default]
    Jobs,
    Profile,
}
