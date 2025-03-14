//! This module defines some types and templates used across the site.

use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{filters, helpers::format_location};

// Templates.

/// Locations selector template.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/locations.html")]
pub(crate) struct Locations {
    pub locations: Vec<Location>,
}

/// Members selector template.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/members.html")]
pub(crate) struct Members {
    pub members: Vec<Member>,
}

/// Projects selector template.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "misc/projects.html")]
pub(crate) struct Projects {
    pub projects: Vec<Project>,
}

// Types.

/// Location information.
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

/// Member information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Member {
    pub member_id: Uuid,
    pub name: String,
    pub level: String,
    pub logo_url: String,
}

/// Project information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Project {
    pub project_id: Uuid,
    pub maturity: String,
    pub name: String,
    pub logo_url: String,
}
