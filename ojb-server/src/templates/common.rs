//! This module defines some common templates used across the site.

use rinja::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{filters, helpers::build_location};

/// Locations.
#[derive(Debug, Clone, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "common/locations.html")]
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

impl Location {
    /// Format the location.
    pub(crate) fn format(&self) -> Option<String> {
        build_location(Some(&self.city), self.state.as_deref(), Some(&self.country))
    }
}
