//! This module defines some templates and types used in the jobs pages.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    PageId,
    auth::User,
    dashboard::employer::jobs::{JobKind, SalaryKind, Workplace},
    filters,
    helpers::{DATE_FORMAT, DATE_FORMAT_3, build_jobboard_image_url, option_is_none_or_default},
    misc::{Foundation, Location, Member, Project},
    pagination::{NavigationLinks, Pagination},
};

// Pages and sections templates.

/// Jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/jobs.html")]
pub(crate) struct JobsPage {
    pub explore_section: ExploreSection,
    pub page_id: PageId,
    pub user: User,
}

/// Explore section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/explore_section.html")]
pub(crate) struct ExploreSection {
    pub filters: Filters,
    pub filters_options: FiltersOptions,
    pub results_section: ResultsSection,
}

/// Results section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/results_section.html")]
pub(crate) struct ResultsSection {
    pub jobs: Vec<JobSummary>,
    pub navigation_links: NavigationLinks,
    pub total: usize,

    pub offset: Option<usize>,
}

/// Job section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/job_section.html")]
pub(crate) struct JobSection {
    pub job: Job,
}

// Types.

/// Filters used in the jobs explore section.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Filters {
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub benefits: Option<Vec<String>>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub date_range: Option<DateRange>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub foundation: Option<String>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub kind: Option<Vec<JobKind>>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub location: Option<Location>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub max_distance: Option<u64>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub offset: Option<usize>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub open_source: Option<usize>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub projects: Option<Vec<String>>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub salary_min: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seniority: Option<Seniority>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub ts_query: Option<String>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub upstream_commitment: Option<usize>,
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub workplace: Option<Vec<Workplace>>,
}

impl Filters {
    /// Check if the filters are empty.
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self == &Filters::default()
    }
}

impl Pagination for Filters {
    fn get_base_hx_url(&self) -> String {
        "/section/jobs/results".to_string()
    }

    fn get_base_url(&self) -> String {
        "/".to_string()
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

/// Date range filter option.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum DateRange {
    LastDay,
    Last3Days,
    Last7Days,
    #[default]
    Last30Days,
}

/// Seniority level filter option.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Seniority {
    Entry,
    Junior,
    Mid,
    Senior,
    Lead,
}

/// Filters options used in the jobs explore section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    pub foundations: Vec<Foundation>,
    pub projects: Vec<Project>,
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
    pub seniority: Option<Seniority>,
    pub skills: Option<Vec<String>>,
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

/// Job details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Job {
    pub description: String,
    pub employer: Employer,
    pub job_id: Uuid,
    pub kind: JobKind,
    pub title: String,
    pub workplace: Workplace,

    pub apply_instructions: Option<String>,
    pub apply_url: Option<String>,
    pub benefits: Option<Vec<String>>,
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
    pub seniority: Option<Seniority>,
    pub skills: Option<Vec<String>>,
    pub tz_end: Option<String>,
    pub tz_start: Option<String>,
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
