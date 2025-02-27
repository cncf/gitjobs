//! This module defines some templates used across the site.

use rinja::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{filters, helpers::format_location};

/// Locations.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/locations.html")]
pub(crate) struct Locations {
    pub locations: Vec<Location>,
}

/// Location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Members.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/members.html")]
pub(crate) struct Members {
    pub members: Vec<Member>,
}

/// Member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Member {
    pub member_id: Uuid,
    pub name: String,
    pub level: String,
    pub logo_url: String,
}

/// Projects.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/projects.html")]
pub(crate) struct Projects {
    pub projects: Vec<Project>,
}

/// Project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Project {
    pub project_id: Uuid,
    pub maturity: String,
    pub name: String,
    pub logo_url: String,
}
