//! This module defines some templates and types used in the jobs pages.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rinja::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::trace;
use uuid::Uuid;

use crate::templates::{
    PageId,
    dashboard::employer::jobs::{JobKind, SalaryKind, Workplace},
    misc::{Location, Member, Project},
    pagination::{NavigationLinks, Pagination},
};

/// Jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/jobs.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct JobsPage {
    pub explore_section: ExploreSection,
    pub logged_in: bool,
    pub page_id: PageId,

    pub name: Option<String>,
    pub username: Option<String>,
}

/// Explore section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/explore_section.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ExploreSection {
    pub filters: Filters,
    pub filters_options: FiltersOptions,
    pub results_section: ResultsSection,
}

/// Filters used in the jobs explore section.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Filters {
    pub benefits: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub kind: Option<Vec<JobKind>>,
    pub limit: Option<usize>,
    pub location_id: Option<Uuid>,
    pub max_distance: Option<u64>,
    pub offset: Option<usize>,
    pub open_source: Option<usize>,
    pub projects: Option<Vec<String>>,
    pub salary_min: Option<u64>,
    pub seniority: Option<String>,
    pub skills: Option<Vec<String>>,
    pub ts_query: Option<String>,
    pub upstream_commitment: Option<usize>,
    pub workplace: Option<Vec<Workplace>>,
}

impl Filters {
    /// Create a new `Filters` instance from the raw query string provided.
    pub(crate) fn new(serde_qs_de: &serde_qs::Config, raw_query: &str) -> Result<Self> {
        let filters: Filters = serde_qs_de.deserialize_str(raw_query)?;

        trace!("{:?}", filters);
        Ok(filters)
    }

    /// Check if the filters are empty.
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self == &Filters::default()
    }
}

impl Pagination for Filters {
    fn get_base_hx_url(&self) -> String {
        "/jobs/section/results".to_string()
    }

    fn get_base_url(&self) -> String {
        "/jobs".to_string()
    }

    fn limit(&self) -> Option<usize> {
        self.limit
    }

    fn offset(&self) -> Option<usize> {
        self.offset
    }

    fn set_offset(&mut self, offset: Option<usize>) {
        self.offset = offset;
    }
}

/// Date range filter options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DateRange {
    LastDay,
    Last3Days,
    Last7Days,
    Last30Days,
}

/// Filters options used in the jobs explore section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    pub benefits: Vec<String>,
    pub projects: Vec<Project>,
    pub skills: Vec<String>,
}

/// Filter option details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FilterOption {
    pub name: String,
    pub value: String,
}

/// Results section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/results_section.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ResultsSection {
    pub jobs: Vec<JobSummary>,
    pub navigation_links: NavigationLinks,
    pub total: usize,

    pub offset: Option<usize>,
}

/// Job summary.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobSummary {
    pub employer: Employer,
    pub job_id: uuid::Uuid,
    pub kind: JobKind,
    pub published_at: DateTime<Utc>,
    pub title: String,
    pub workplace: Workplace,

    pub location: Option<Location>,
    pub open_source: Option<i32>,
    pub projects: Option<Vec<Project>>,
    pub salary: Option<i64>,
    pub salary_currency: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_period: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub upstream_commitment: Option<i32>,
}

/// Employer details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Employer {
    pub company: String,
    pub employer_id: Uuid,

    pub description: Option<String>,
    pub logo_id: Option<Uuid>,
    pub member: Option<Member>,
    pub website_url: Option<String>,
}

/// Job page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/job.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct JobPage {
    pub job: Job,
    pub logged_in: bool,
    pub page_id: PageId,

    pub name: Option<String>,
    pub username: Option<String>,
}

/// Job details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Job {
    pub description: String,
    pub employer: Employer,
    pub title: String,
    pub kind: JobKind,
    pub workplace: Workplace,

    pub apply_instructions: Option<String>,
    pub apply_url: Option<String>,
    pub benefits: Option<Vec<String>>,
    pub job_id: Option<Uuid>,
    pub location: Option<Location>,
    pub open_source: Option<i32>,
    pub projects: Option<Vec<Project>>,
    pub published_at: Option<DateTime<Utc>>,
    pub qualifications: Option<String>,
    pub responsibilities: Option<String>,
    pub salary: Option<i64>,
    pub salary_currency: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_period: Option<String>,
    pub skills: Option<Vec<String>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub upstream_commitment: Option<i32>,
}

impl Job {
    /// Get the salary kind of the job.
    #[allow(dead_code)]
    pub(crate) fn salary_kind(&self) -> SalaryKind {
        if self.salary_min.is_some() && self.salary_max.is_some() {
            SalaryKind::Range
        } else {
            SalaryKind::Fixed
        }
    }
}
