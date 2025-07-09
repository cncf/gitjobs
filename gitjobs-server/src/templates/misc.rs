//! This module defines types and templates used across the site for various features.

use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{Config, PageId, auth::User, filters, helpers::format_location};

// Templates.

/// Template for the 404 Not Found error page.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/not_found.html")]
pub(crate) struct NotFoundPage {
    /// Server configuration.
    pub cfg: Config,
    /// Identifier for the current page.
    pub page_id: PageId,
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
}

/// Template for the user menu section in the UI.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "misc/user_menu_section.html")]
pub(crate) struct UserMenuSection {
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
}

// Types.

/// Information about a certification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Certification {
    /// Unique identifier for the certification.
    pub certification_id: Uuid,
    /// Full name of the certification.
    pub name: String,
    /// Provider of the certification.
    pub provider: String,
    /// Short name or abbreviation.
    pub short_name: String,

    /// Description of the certification.
    pub description: Option<String>,
    /// Logo URL for the certification.
    pub logo_url: Option<String>,
    /// URL to certification information.
    pub url: Option<String>,
}

/// Information about a foundation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Foundation {
    /// Name of the foundation.
    pub name: String,
}

/// Information about a location.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Location {
    /// Unique identifier for the location.
    pub location_id: Uuid,
    /// City name.
    pub city: String,
    /// Country name.
    pub country: String,

    /// State or region, if any.
    pub state: Option<String>,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format_location(Some(&self.city), self.state.as_deref(), Some(&self.country))
                .expect("output to be some")
        )
    }
}

/// Information about a member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Member {
    /// Unique identifier for the member.
    pub member_id: Uuid,
    /// Foundation name.
    pub foundation: String,
    /// Membership level.
    pub level: String,
    /// Logo URL for the member.
    pub logo_url: String,
    /// Name of the member.
    pub name: String,
}

/// Information about a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Project {
    /// Unique identifier for the project.
    pub project_id: Uuid,
    /// Foundation name.
    pub foundation: String,
    /// Logo URL for the project.
    pub logo_url: String,
    /// Maturity level of the project.
    pub maturity: String,
    /// Name of the project.
    pub name: String,
}
