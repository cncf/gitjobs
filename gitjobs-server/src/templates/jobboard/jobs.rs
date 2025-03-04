//! This module defines some templates and types used in the jobs page.

use anyhow::Result;
use rinja::Template;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::templates::{
    PageId,
    dashboard::employer::jobs::{Job, JobKind, Workplace},
    filters,
};

/// Jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/page.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct Page {
    pub explore_section: ExploreSection,
    pub logged_in: bool,
    pub page_id: PageId,

    pub name: Option<String>,
    pub username: Option<String>,
}

/// Explore section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/explore.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ExploreSection {
    pub filters: Filters,
    pub filters_options: FiltersOptions,
    pub results_section: ResultsSection,
}

/// Results section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/results.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ResultsSection {
    pub jobs: Vec<Job>,
    pub total: usize,

    pub offset: Option<usize>,
}

/// Filters used in the jobs explore section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")],
    Vec => #[serde(default)],
)]
pub(crate) struct Filters {
    pub kind: Vec<JobKind>,
    pub workplace: Vec<Workplace>,

    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub ts_query: Option<String>,
}

impl Filters {
    /// Create a new `Filters` instance from the raw query string provided.
    pub(crate) fn new(raw_query: &str) -> Result<Self> {
        let filters: Filters = serde_qs::from_str(raw_query)?;

        trace!("{:?}", filters);
        Ok(filters)
    }

    /// Convert the filters to a raw query string.
    #[allow(dead_code)]
    fn to_raw_query(&self) -> Result<String> {
        serde_qs::to_string(self).map_err(Into::into)
    }
}

/// Filters options used in the jobs explore section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    pub kind: Vec<FilterOption>,
    pub workplace: Vec<FilterOption>,
}

/// Filter option details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FilterOption {
    pub name: String,
    pub value: String,
}
