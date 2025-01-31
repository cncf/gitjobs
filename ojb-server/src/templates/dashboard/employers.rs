//! This module defines some templates and types used to manage employers

use rinja::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{filters, helpers::build_location};

/// Add employer page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employers/add.html")]
pub(crate) struct AddPage {}

/// Employer initial setup page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employers/initial_setup.html")]
pub(crate) struct InitialSetupPage {}

/// Update employer page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employers/update.html")]
pub(crate) struct UpdatePage {
    pub employer_details: EmployerDetails,
}

/// Employer summary.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EmployerSummary {
    pub employer_id: Uuid,
    pub company: String,
}

/// Employer details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EmployerDetails {
    pub company: String,
    pub description: String,
    pub public: bool,

    pub city: Option<String>,
    pub country: Option<String>,
    pub location_id: Option<Uuid>,
    pub logo_url: Option<String>,
    pub state: Option<String>,
    pub website_url: Option<String>,
}

impl EmployerDetails {
    /// Get the location of the employer.
    #[allow(dead_code)]
    pub(crate) fn location(&self) -> Option<String> {
        build_location(
            self.city.as_deref(),
            self.state.as_deref(),
            self.country.as_deref(),
        )
    }
}
