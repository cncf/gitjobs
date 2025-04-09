//! This module defines some templates and types used in the moderator
//! dashboard jobs pages.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{helpers::DATE_FORMAT, misc::Member};

// Pages templates.

/// Pending jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/moderator/pending_jobs.html")]
pub(crate) struct PendingPage {
    pub jobs: Vec<JobSummary>,
}

// Types.

/// Job summary.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobSummary {
    pub created_at: DateTime<Utc>,
    pub employer: Employer,
    pub job_id: uuid::Uuid,
    pub title: String,
}

/// Employer details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Employer {
    pub company: String,
    pub employer_id: Uuid,

    pub logo_id: Option<Uuid>,
    pub member: Option<Member>,
    pub website_url: Option<String>,
}
