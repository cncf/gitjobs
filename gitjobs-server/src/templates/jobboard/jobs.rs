//! Templates and types for job board pages, sections, and job-related data.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    Config, PageId,
    auth::User,
    dashboard::employer::jobs::{JobKind, SalaryKind, Workplace},
    filters,
    helpers::{DATE_FORMAT, DATE_FORMAT_3, build_jobboard_image_url, option_is_none_or_default},
    misc::{Certification, Foundation, Location, Member, Project},
    pagination::{NavigationLinks, Pagination},
};

// Pages and sections templates.

/// Template for the main jobs page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/jobs.html")]
pub(crate) struct JobsPage {
    /// Server configuration.
    pub cfg: Config,
    /// Explore section containing filters and results.
    pub explore_section: ExploreSection,
    /// Identifier for the current page.
    pub page_id: PageId,
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
}

/// Template for the explore section, containing filters and results.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/explore_section.html")]
pub(crate) struct ExploreSection {
    /// Filters applied to the job search.
    pub filters: Filters,
    /// Available options for filters.
    pub filters_options: FiltersOptions,
    /// Section displaying the results.
    pub results_section: ResultsSection,
}

/// Template for the results section, showing job summaries and navigation.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/results_section.html")]
pub(crate) struct ResultsSection {
    /// List of job summaries.
    pub jobs: Vec<JobSummary>,
    /// Navigation links for pagination.
    pub navigation_links: NavigationLinks,
    /// Total number of jobs found.
    pub total: usize,

    /// Offset for pagination.
    pub offset: Option<usize>,
}

/// Template for a single job section, displaying job details.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/job_section.html")]
pub(crate) struct JobSection {
    /// Base URL for job links and assets.
    pub base_url: String,
    /// Full job details.
    pub job: Job,
}

// Types.

/// Filters for searching and narrowing down job listings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Filters {
    /// List of required benefits.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub benefits: Option<Vec<String>>,
    /// Date range for job posting.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub date_range: Option<DateRange>,
    /// Foundation filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub foundation: Option<String>,
    /// Job kinds to filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub kind: Option<Vec<JobKind>>,
    /// Limit for pagination.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub limit: Option<usize>,
    /// Location filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub location: Option<Location>,
    /// Maximum distance from location.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub max_distance: Option<u64>,
    /// Foundation membership filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub membership: Option<String>,
    /// Offset for pagination.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub offset: Option<usize>,
    /// Open source filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub open_source: Option<usize>,
    /// Project filters.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub projects: Option<Vec<JobProject>>,
    /// Minimum salary filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub salary_min: Option<u64>,
    /// Seniority filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seniority: Option<Seniority>,
    /// Skills filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub skills: Option<Vec<String>>,
    /// Sorting option.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub sort: Option<Sort>,
    /// Full-text search query.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub ts_query: Option<String>,
    /// Upstream commitment filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub upstream_commitment: Option<usize>,
    /// Workplace types filter.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub workplace: Option<Vec<Workplace>>,
}

impl Filters {
    /// Returns true if all filter fields are unset or default.
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

/// Date range options for filtering job postings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum DateRange {
    /// Last day.
    LastDay,
    /// Last 3 days.
    Last3Days,
    /// Last 7 days.
    Last7Days,
    /// Last 30 days (default).
    #[default]
    Last30Days,
}

/// Project filter for jobs, specifying foundation and project name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct JobProject {
    /// Foundation name.
    pub foundation: String,
    /// Project name.
    pub name: String,
}

/// Seniority level filter for job listings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Seniority {
    /// Entry level.
    Entry,
    /// Junior level.
    Junior,
    /// Mid level.
    Mid,
    /// Senior level.
    Senior,
    /// Lead level.
    Lead,
}

/// Sorting options for job search results.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Sort {
    /// Sort by date (default).
    #[default]
    Date,
    /// Sort by open source commitment.
    OpenSource,
    /// Sort by salary.
    Salary,
    /// Sort by upstream commitment.
    UpstreamCommitment,
}

/// Options for filters in the explore section, such as available foundations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    /// List of available foundations.
    pub foundations: Vec<Foundation>,
}

/// Summary information for a job, used in job listings.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobSummary {
    /// Employer information.
    pub employer: Employer,
    /// Unique identifier for the job.
    pub job_id: uuid::Uuid,
    /// Kind of job.
    pub kind: JobKind,
    /// Timestamp when the job was published.
    pub published_at: DateTime<Utc>,
    /// Title of the job.
    pub title: String,
    /// Workplace type for the job.
    pub workplace: Workplace,

    /// Location of the job, if specified.
    pub location: Option<Location>,
    /// Open source status, if specified.
    pub open_source: Option<i32>,
    /// List of related projects, if any.
    pub projects: Option<Vec<Project>>,
    /// Salary value, if specified.
    pub salary: Option<i64>,
    /// Salary currency, if specified.
    pub salary_currency: Option<String>,
    /// Minimum salary, if specified.
    pub salary_min: Option<i64>,
    /// Maximum salary, if specified.
    pub salary_max: Option<i64>,
    /// Salary period, if specified.
    pub salary_period: Option<String>,
    /// Seniority level, if specified.
    pub seniority: Option<Seniority>,
    /// List of required skills, if any.
    pub skills: Option<Vec<String>>,
    /// Timestamp when the job was last updated, if any.
    pub updated_at: Option<DateTime<Utc>>,
    /// Upstream commitment, if specified.
    pub upstream_commitment: Option<i32>,
}

/// Employer information for job listings and details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Employer {
    /// Name of the company.
    pub company: String,
    /// Unique identifier for the employer.
    pub employer_id: Uuid,
    /// Description of the employer, if any.
    pub description: Option<String>,
    /// Logo identifier, if any.
    pub logo_id: Option<Uuid>,
    /// Members associated with the employer, if any.
    pub members: Option<Vec<Member>>,
    /// Website URL, if any.
    pub website_url: Option<String>,
}

/// Full job details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Job {
    /// Job description.
    pub description: String,
    /// Employer information.
    pub employer: Employer,
    /// Unique identifier for the job.
    pub job_id: Uuid,
    /// Kind of job.
    pub kind: JobKind,
    /// Title of the job.
    pub title: String,
    /// Workplace type for the job.
    pub workplace: Workplace,

    /// Application instructions, if any.
    pub apply_instructions: Option<String>,
    /// Application URL, if any.
    pub apply_url: Option<String>,
    /// List of benefits, if any.
    pub benefits: Option<Vec<String>>,
    /// Desired certifications, if any.
    pub certifications: Option<Vec<Certification>>,
    /// Location of the job, if specified.
    pub location: Option<Location>,
    /// Open source status, if specified.
    pub open_source: Option<i32>,
    /// List of related projects, if any.
    pub projects: Option<Vec<Project>>,
    /// Timestamp when the job was published, if any.
    pub published_at: Option<DateTime<Utc>>,
    /// Required qualifications, if any.
    pub qualifications: Option<String>,
    /// Responsibilities, if any.
    pub responsibilities: Option<String>,
    /// Salary value, if specified.
    pub salary: Option<i64>,
    /// Salary currency, if specified.
    pub salary_currency: Option<String>,
    /// Minimum salary, if specified.
    pub salary_min: Option<i64>,
    /// Maximum salary, if specified.
    pub salary_max: Option<i64>,
    /// Salary period, if specified.
    pub salary_period: Option<String>,
    /// Seniority level, if specified.
    pub seniority: Option<Seniority>,
    /// List of required skills, if any.
    pub skills: Option<Vec<String>>,
    /// Timezone end, if any.
    pub tz_end: Option<String>,
    /// Timezone start, if any.
    pub tz_start: Option<String>,
    /// Timestamp when the job was last updated, if any.
    pub updated_at: Option<DateTime<Utc>>,
    /// Upstream commitment, if specified.
    pub upstream_commitment: Option<i32>,
}

impl Job {
    /// Determines if the job salary is a fixed value or a range.
    #[allow(dead_code)]
    pub(crate) fn salary_kind(&self) -> SalaryKind {
        if self.salary_min.is_some() && self.salary_max.is_some() {
            SalaryKind::Range
        } else {
            SalaryKind::Fixed
        }
    }
}
