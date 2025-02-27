//! This module defines some templates and types used to manage employers.

use rinja::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    filters,
    helpers::format_location,
    misc::{Location, Member},
};

/// Add employer page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/employers/add.html")]
pub(crate) struct AddPage {}

/// Employer initial setup page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/employers/initial_setup.html")]
pub(crate) struct InitialSetupPage {}

/// Update employer page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/employers/update.html")]
pub(crate) struct UpdatePage {
    pub employer: Employer,
}

/// Employer summary.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EmployerSummary {
    pub employer_id: Uuid,
    pub company: String,

    pub logo_id: Option<Uuid>,
}

/// Employer details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Employer {
    pub company: String,
    pub description: String,
    pub public: bool,

    pub location: Option<Location>,
    pub logo_id: Option<Uuid>,
    pub member: Option<Member>,
    pub website_url: Option<String>,
}
