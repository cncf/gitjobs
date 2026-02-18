//! Templates and types for managing employers in the employer dashboard.

use askama::Template;
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    templates::{
        filters,
        helpers::build_dashboard_image_url,
        misc::{Foundation, Location, Member},
    },
    validation::{MAX_LEN_DESCRIPTION, MAX_LEN_ENTITY_NAME, MAX_LEN_L, trimmed_non_empty},
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct Employer {
    /// Company name.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub company: String,
    /// Company description.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DESCRIPTION))]
    pub description: String,
    /// Whether the employer profile is public.
    #[garde(skip)]
    pub public: bool,

    /// Location of the employer, if specified.
    #[garde(skip)]
    pub location: Option<Location>,
    /// Logo image identifier, if available.
    #[garde(skip)]
    pub logo_id: Option<Uuid>,
    /// Associated members, if any.
    #[garde(skip)]
    pub members: Option<Vec<Member>>,
    /// Website URL, if provided.
    #[garde(url, length(max = MAX_LEN_L))]
    pub website_url: Option<String>,
}
