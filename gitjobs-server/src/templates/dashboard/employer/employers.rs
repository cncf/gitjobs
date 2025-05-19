//! Templates and types for managing employers in the employer dashboard.

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    filters,
    helpers::build_dashboard_image_url,
    misc::{Foundation, Location, Member},
};

// Pages templates.

/// Add employer page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/employers/add.html")]
pub(crate) struct AddPage {
    /// List of available foundations for employer association.
    pub foundations: Vec<Foundation>,
}

/// Employer initial setup page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/employers/initial_setup.html")]
pub(crate) struct InitialSetupPage {}

/// Update employer page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/employers/update.html")]
pub(crate) struct UpdatePage {
    /// Employer details to update.
    pub employer: Employer,
    /// List of available foundations for employer association.
    pub foundations: Vec<Foundation>,
}

// Types.

/// Employer summary information for dashboard listings.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EmployerSummary {
    /// Unique identifier for the employer.
    pub employer_id: Uuid,
    /// Company name.
    pub company: String,
    /// Logo image identifier, if available.
    pub logo_id: Option<Uuid>,
}

/// Employer details for dashboard management.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Employer {
    /// Company name.
    pub company: String,
    /// Company description.
    pub description: String,
    /// Whether the employer profile is public.
    pub public: bool,
    /// Location of the employer, if specified.
    pub location: Option<Location>,
    /// Logo image identifier, if available.
    pub logo_id: Option<Uuid>,
    /// Associated member information, if any.
    pub member: Option<Member>,
    /// Website URL, if provided.
    pub website_url: Option<String>,
}
