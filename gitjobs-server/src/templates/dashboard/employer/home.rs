//! Templates and types for the employer dashboard home page.

use askama::Template;
use axum_messages::{Level, Message};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{
    Config, PageId,
    auth::{self, User},
    dashboard::employer,
    filters,
    helpers::{build_dashboard_image_url, find_employer},
};

// Pages templates.

/// Home page template for the employer dashboard.
#[derive(Debug, Clone, Template)]
#[template(path = "dashboard/employer/home.html")]
pub(crate) struct Page {
    /// Application configuration.
    pub cfg: Config,
    /// Main content section for the page.
    pub content: Content,
    /// List of employers for the user.
    pub employers: Vec<employer::employers::EmployerSummary>,
    /// List of messages to display.
    pub messages: Vec<Message>,
    /// Page identifier.
    pub page_id: PageId,
    /// Number of pending team invitations.
    pub pending_invitations: usize,
    /// Authenticated user information.
    pub user: User,
    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
    /// Selected employer's unique identifier, if any.
    pub selected_employer_id: Option<Uuid>,
}

// Types.

/// Content section for the employer dashboard home page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Content {
    /// User account page.
    Account(auth::UpdateUserPage),
    /// Applications list page.
    Applications(employer::applications::ApplicationsPage),
    /// Initial setup page for employer profile.
    EmployerInitialSetup(employer::employers::InitialSetupPage),
    /// Team invitations list page.
    Invitations(employer::team::UserInvitationsListPage),
    /// Jobs list page.
    Jobs(employer::jobs::ListPage),
    /// Employer profile update page.
    Profile(employer::employers::UpdatePage),
    /// Team members list page.
    Team(employer::team::MembersListPage),
}

impl Content {
    /// Check if the content is the account page.
    fn is_account(&self) -> bool {
        matches!(self, Content::Account(_))
    }

    /// Check if the content is the applications page.
    fn is_applications(&self) -> bool {
        matches!(self, Content::Applications(_))
    }

    /// Check if the content is the invitations page.
    fn is_invitations(&self) -> bool {
        matches!(self, Content::Invitations(_))
    }

    /// Check if the content is the jobs page.
    fn is_jobs(&self) -> bool {
        matches!(self, Content::Jobs(_))
    }

    /// Check if the content is the profile page.
    fn is_profile(&self) -> bool {
        matches!(self, Content::Profile(_))
    }

    /// Check if the content is the team page.
    fn is_team(&self) -> bool {
        matches!(self, Content::Team(_))
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Account(template) => write!(f, "{}", template.render()?),
            Content::Applications(template) => write!(f, "{}", template.render()?),
            Content::EmployerInitialSetup(template) => write!(f, "{}", template.render()?),
            Content::Invitations(template) => write!(f, "{}", template.render()?),
            Content::Jobs(template) => write!(f, "{}", template.render()?),
            Content::Profile(template) => write!(f, "{}", template.render()?),
            Content::Team(template) => write!(f, "{}", template.render()?),
        }
    }
}

/// Tab selection for the employer dashboard home page.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Tab {
    /// User account tab.
    Account,
    /// Applications tab.
    Applications,
    /// Employer initial setup tab.
    EmployerInitialSetup,
    /// Team invitations tab.
    Invitations,
    /// Jobs tab (default).
    #[default]
    Jobs,
    /// Employer profile tab.
    Profile,
    /// Team members tab.
    Team,
}
