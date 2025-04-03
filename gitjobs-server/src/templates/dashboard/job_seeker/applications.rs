//! This module defines some templates and types used in the applications page.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{dashboard::employer::jobs::Workplace, helpers::DATE_FORMAT, misc::Location};

// Pages templates.

/// Applications page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/job_seeker/applications/list.html")]
pub(crate) struct ApplicationsPage {
    pub applications: Vec<Application>,
}

// Types.

/// Application information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Application {
    pub application_id: Uuid,
    pub applied_at: DateTime<Utc>,
    pub job_id: Uuid,
    pub job_title: String,
    pub job_workplace: Workplace,

    pub job_location: Option<Location>,
}
