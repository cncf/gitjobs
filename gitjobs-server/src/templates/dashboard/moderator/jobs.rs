//! Templates and types for moderator dashboard jobs pages.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    helpers::{DATE_FORMAT, DATE_FORMAT_3},
    misc::Member,
};

// Pages templates.

/// Template for the live jobs page in the moderator dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/moderator/live_jobs.html")]
pub(crate) struct LivePage {
    /// List of live jobs.
    pub jobs: Vec<JobSummary>,
}

/// Template for the pending jobs page in the moderator dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/moderator/pending_jobs.html")]
pub(crate) struct PendingPage {
    /// List of pending jobs.
    pub jobs: Vec<JobSummary>,
}

// Types.

/// Summary information for a job, used in moderator dashboard listings.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobSummary {
    /// Timestamp when the job was created.
    pub created_at: DateTime<Utc>,
    /// Employer information for the job.
    pub employer: Employer,
    /// Unique identifier for the job.
    pub job_id: uuid::Uuid,
    /// Title of the job.
    pub title: String,
}

/// Employer information for job summaries in the moderator dashboard.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Employer {
    /// Name of the company.
    pub company: String,
    /// Unique identifier for the employer.
    pub employer_id: Uuid,
    /// Optional logo identifier for the employer.
    pub logo_id: Option<Uuid>,
    /// Optional member information for the employer.
    pub member: Option<Member>,
    /// Optional website URL for the employer.
    pub website_url: Option<String>,
}
