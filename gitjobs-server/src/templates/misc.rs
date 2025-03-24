//! This module defines some types and templates used across the site.

use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{PageId, auth::User, helpers::format_location};

// Templates.

/// Members selector template.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/members.html")]
pub(crate) struct Members {
    pub members: Vec<Member>,
}

/// Not found page template.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/not_found.html")]
pub(crate) struct NotFoundPage {
    pub page_id: PageId,
    pub user: User,
}

/// Projects selector template.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/projects.html")]
pub(crate) struct Projects {
    pub projects: Vec<Project>,
}

/// User menu section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "misc/user_menu_section.html")]
pub(crate) struct UserMenuSection {
    pub user: User,
}

// Types.

/// Location information.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Location {
    pub location_id: Uuid,
    pub city: String,
    pub country: String,

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

/// Member information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Member {
    pub member_id: Uuid,
    pub foundation: String,
    pub level: String,
    pub logo_url: String,
    pub name: String,
}

/// Project information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Project {
    pub project_id: Uuid,
    pub foundation: String,
    pub logo_url: String,
    pub maturity: String,
    pub name: String,
}
