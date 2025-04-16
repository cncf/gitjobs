//! This module defines some templates and types used in the embed page.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{
    dashboard::employer::jobs::{JobKind, Workplace},
    filters,
    helpers::{DATE_FORMAT_3, build_jobboard_image_url},
    jobboard::jobs::JobSummary,
};

/// Embed page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/embed/page.html")]
pub(crate) struct Page {
    pub base_url: String,
    pub jobs: Vec<JobSummary>,
}
