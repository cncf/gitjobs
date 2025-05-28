//! Templates and types for the job seeker applications page.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::{
    dashboard::employer::jobs::{JobStatus, Workplace},
    helpers::DATE_FORMAT,
    misc::Location,
};

// Pages templates.

/// Applications page template for job seeker dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/job_seeker/applications/list.html")]
pub(crate) struct ApplicationsPage {
    /// List of job applications for the job seeker.
    pub applications: Vec<Application>,
}

// Types.

/// Represents a job application entry for the job seeker dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Application {
    /// Unique identifier for the application.
    pub application_id: Uuid,
    /// Timestamp when the application was submitted.
    pub applied_at: DateTime<Utc>,
    /// Unique identifier for the job.
    pub job_id: Uuid,
    /// Status of the job.
    pub job_status: JobStatus,
    /// Title of the job applied for.
    pub job_title: String,
    /// Workplace type for the job.
    pub job_workplace: Workplace,

    /// Location of the job, if specified.
    pub job_location: Option<Location>,
}
