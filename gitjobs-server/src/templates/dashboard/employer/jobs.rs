//! Templates and types for the employer dashboard jobs page.

use askama::Template;
use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    templates::{
        dashboard::employer::employers::Employer,
        filters,
        helpers::{DATE_FORMAT, build_dashboard_image_url, format_location, normalize, normalize_salary},
        jobboard::jobs::Seniority,
        misc::{Certification, Foundation, Location, Project},
    },
    validation::{
        MAX_LEN_DESCRIPTION, MAX_LEN_DESCRIPTION_SHORT, MAX_LEN_ENTITY_NAME, MAX_LEN_L, MAX_LEN_S,
        trimmed_non_empty, trimmed_non_empty_opt, trimmed_non_empty_tag_vec, trimmed_non_empty_vec,
    },
};

// Pages templates.

/// Add job page template for the employer dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/add.html")]
pub(crate) struct AddPage {
    /// List of available certifications for job requirements.
    pub certifications: Vec<Certification>,
    /// List of available foundations for job association.
    pub foundations: Vec<Foundation>,
}

/// Jobs list page template for the employer dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/list.html")]
pub(crate) struct ListPage {
    /// List of jobs for the employer.
    pub jobs: Vec<JobSummary>,
}

/// Job preview page template for the employer dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/preview.html")]
pub(crate) struct PreviewPage {
    /// Employer information for the job.
    pub employer: Employer,
    /// Job details to preview.
    pub job: Job,
}

/// Update job page template for the employer dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/update.html")]
pub(crate) struct UpdatePage {
    /// List of available certifications for job requirements.
    pub certifications: Vec<Certification>,
    /// List of available foundations for job association.
    pub foundations: Vec<Foundation>,
    /// Job details to update.
    pub job: Job,
}

// Types.

/// Job summary information for employer dashboard listings.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct JobSummary {
    /// Unique identifier for the job.
    pub job_id: uuid::Uuid,
    /// Timestamp when the job was created.
    pub created_at: DateTime<Utc>,
    /// Job title.
    pub title: String,
    /// Current status of the job.
    pub status: JobStatus,
    /// Workplace type for the job.
    pub workplace: Workplace,

    /// Timestamp when the job was archived, if applicable.
    pub archived_at: Option<DateTime<Utc>>,
    /// City where the job is located, if specified.
    pub city: Option<String>,
    /// Country where the job is located, if specified.
    pub country: Option<String>,
    /// Timestamp when the job was published, if applicable.
    pub published_at: Option<DateTime<Utc>>,
    /// Notes from job review, if any.
    pub review_notes: Option<String>,
}

/// Job details for the employer dashboard.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub(crate) struct Job {
    /// Job description text.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DESCRIPTION))]
    pub description: String,
    /// Current status of the job.
    #[garde(skip)]
    pub status: JobStatus,
    /// Job title.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_ENTITY_NAME))]
    pub title: String,
    /// Kind of job (full-time, part-time, etc.).
    #[garde(skip)]
    pub kind: JobKind,
    /// Workplace type for the job.
    #[garde(skip)]
    pub workplace: Workplace,

    /// Application instructions, if provided.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION))]
    pub apply_instructions: Option<String>,
    /// External application URL, if provided.
    #[garde(url, length(max = MAX_LEN_L))]
    pub apply_url: Option<String>,
    /// List of job benefits, if any.
    #[garde(custom(trimmed_non_empty_vec))]
    pub benefits: Option<Vec<String>>,
    /// Desired certifications, if any.
    #[garde(skip)]
    pub certifications: Option<Vec<Certification>>,
    /// Unique identifier for the job, if available.
    #[garde(skip)]
    pub job_id: Option<Uuid>,
    /// Location details for the job, if specified.
    #[garde(skip)]
    pub location: Option<Location>,
    /// Open source commitment level, if specified.
    #[garde(skip)]
    pub open_source: Option<i32>,
    /// Related projects, if any.
    #[garde(skip)]
    pub projects: Option<Vec<Project>>,
    /// Timestamp when the job was published, if applicable.
    #[garde(skip)]
    pub published_at: Option<DateTime<Utc>>,
    /// Required qualifications, if any.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION))]
    pub qualifications: Option<String>,
    /// Job responsibilities, if any.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION))]
    pub responsibilities: Option<String>,
    /// Notes from job review, if any.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    pub review_notes: Option<String>,
    /// Salary amount, if specified.
    #[garde(skip)]
    pub salary: Option<i64>,
    /// Salary normalized to USD per year, if available.
    #[garde(skip)]
    pub salary_usd_year: Option<i64>,
    /// Currency of the salary, if specified.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_S))]
    pub salary_currency: Option<String>,
    /// Minimum salary, if specified.
    #[garde(skip)]
    pub salary_min: Option<i64>,
    /// Minimum salary normalized to USD per year, if available.
    #[garde(skip)]
    pub salary_min_usd_year: Option<i64>,
    /// Maximum salary, if specified.
    #[garde(skip)]
    pub salary_max: Option<i64>,
    /// Maximum salary normalized to USD per year, if available.
    #[garde(skip)]
    pub salary_max_usd_year: Option<i64>,
    /// Salary period (e.g., year, month, week, day, hour), if specified.
    #[garde(skip)]
    pub salary_period: Option<String>,
    /// Seniority level for the job, if specified.
    #[garde(skip)]
    pub seniority: Option<Seniority>,
    /// List of required or desired skills, if any.
    #[garde(custom(trimmed_non_empty_tag_vec))]
    pub skills: Option<Vec<String>>,
    /// End of timezone range, if specified.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_S))]
    pub tz_end: Option<String>,
    /// Start of timezone range, if specified.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_S))]
    pub tz_start: Option<String>,
    /// Timestamp when the job was last updated, if available.
    #[garde(skip)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Upstream commitment level, if specified.
    #[garde(skip)]
    pub upstream_commitment: Option<i32>,
}

impl Job {
    /// Normalize some fields.
    pub(crate) async fn normalize(&mut self) {
        // Benefits
        if let Some(benefits) = &mut self.benefits {
            for benefit in benefits.iter_mut() {
                *benefit = normalize(benefit);
            }
        }

        // Salary (to USD yearly)
        let (currency, period) = (self.salary_currency.as_ref(), self.salary_period.as_ref());
        self.salary_usd_year = normalize_salary(self.salary, currency, period).await;
        self.salary_min_usd_year = normalize_salary(self.salary_min.or(self.salary), currency, period).await;
        self.salary_max_usd_year = normalize_salary(self.salary_max.or(self.salary), currency, period).await;

        // Skills
        if let Some(skills) = &mut self.skills {
            for skill in skills.iter_mut() {
                *skill = normalize(skill);
            }
        }
    }

    /// Get the salary kind of the job.
    pub(crate) fn salary_kind(&self) -> SalaryKind {
        if self.salary_min.is_some() && self.salary_max.is_some() {
            SalaryKind::Range
        } else {
            SalaryKind::Fixed
        }
    }
}

/// Statistics for a specific job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobStats {
    /// Daily search appearances for the last month.
    /// Each entry is a tuple of (`timestamp_ms`, count).
    pub search_appearances_daily: Option<Vec<(u64, u64)>>,
    /// Total search appearances in the last month.
    pub search_appearances_total_last_month: u64,
    /// Daily views for the last month.
    /// Each entry is a tuple of (`timestamp_ms`, count).
    pub views_daily: Option<Vec<(u64, u64)>>,
    /// Total views in the last month.
    pub views_total_last_month: u64,
}

/// Job status for employer dashboard jobs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum JobStatus {
    /// Job is archived and not visible to users.
    Archived,
    /// Job soft deleted and not visible to users.
    Deleted,
    /// Job is a draft and not yet published.
    #[default]
    Draft,
    /// Job is pending approval by moderators.
    PendingApproval,
    /// Job is published and visible to users.
    Published,
    /// Job was rejected by moderators.
    Rejected,
}

/// Job kind for employer dashboard jobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum JobKind {
    /// Contract position.
    Contractor,
    /// Internship position.
    Internship,
    /// Full-time position.
    FullTime,
    /// Part-time position.
    PartTime,
}

/// Salary kind for employer dashboard jobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum SalaryKind {
    /// Fixed salary amount.
    Fixed,
    /// Salary is a range between min and max.
    Range,
}

/// Workplace type for employer dashboard jobs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Workplace {
    /// Hybrid workplace (mix of remote and on-site).
    Hybrid,
    /// On-site workplace (default).
    #[default]
    OnSite,
    /// Fully remote workplace.
    Remote,
}
