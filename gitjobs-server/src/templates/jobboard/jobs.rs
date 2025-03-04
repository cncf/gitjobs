//! This module defines some templates and types used in the jobs page.

use rinja::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{PageId, filters};

/// Jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/page.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct Page {
    pub logged_in: bool,
    pub page_id: PageId,

    pub name: Option<String>,
    pub username: Option<String>,
}

/// Explore section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/explore.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ExploreSection {}

/// Results section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/results.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ResultsSection {}
