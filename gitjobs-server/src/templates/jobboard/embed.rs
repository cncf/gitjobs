//! Templates and types for job board embed pages and cards.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{
    dashboard::employer::jobs::{JobKind, Workplace},
    filters,
    helpers::{DATE_FORMAT_3, build_jobboard_image_url},
    jobboard::jobs::{Job, JobSummary},
};

/// Template for the jobs page embed, showing a list of job summaries.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/embed/jobs_page.html")]
pub(crate) struct JobsPage {
    /// Base URL for job links and assets.
    pub base_url: String,
    /// List of jobs to display.
    pub jobs: Vec<JobSummary>,
}

/// Template for a single job card embed, rendered as SVG.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/embed/job_card.svg")]
pub(crate) struct JobCard {
    /// Base URL for job links.
    pub base_url: String,

    /// Job data to display in the card.
    pub job: Option<Job>,
}
