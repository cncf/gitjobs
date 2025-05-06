//! This module defines some templates and types used in the embeds.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{
    dashboard::employer::jobs::{JobKind, Workplace},
    filters,
    helpers::{DATE_FORMAT_3, build_jobboard_image_url},
    jobboard::jobs::{Job, JobSummary},
};

/// Jobs page embed template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/embed/jobs_page.html")]
pub(crate) struct JobsPage {
    pub base_url: String,
    pub jobs: Vec<JobSummary>,
}

/// Job card embed template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/embed/job_card.svg")]
pub(crate) struct JobCard {
    pub base_url: String,
    pub job: Option<Job>,
}
