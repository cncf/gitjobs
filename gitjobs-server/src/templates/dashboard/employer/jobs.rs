//! Templates and types for the employer dashboard jobs page.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    dashboard::employer::employers::Employer,
    filters,
    helpers::{DATE_FORMAT, build_dashboard_image_url, format_location, normalize, normalize_salary},
    jobboard::jobs::Seniority,
    misc::{Foundation, Location, Project},
};

// Pages templates.

/// Add job page template for the employer dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/add.html")]
pub(crate) struct AddPage {
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Job {
    /// Job description text.
    pub description: String,
    /// Current status of the job.
    pub status: JobStatus,
    /// Job title.
    pub title: String,
    /// Kind of job (full-time, part-time, etc.).
    pub kind: JobKind,
    /// Workplace type for the job.
    pub workplace: Workplace,

    /// Application instructions, if provided.
    pub apply_instructions: Option<String>,
    /// External application URL, if provided.
    pub apply_url: Option<String>,
    /// List of job benefits, if any.
    pub benefits: Option<Vec<String>>,
    /// Unique identifier for the job, if available.
    pub job_id: Option<Uuid>,
    /// Location details for the job, if specified.
    pub location: Option<Location>,
    /// Open source commitment level, if specified.
    pub open_source: Option<i32>,
    /// Related projects, if any.
    pub projects: Option<Vec<Project>>,
    /// Timestamp when the job was published, if applicable.
    pub published_at: Option<DateTime<Utc>>,
    /// Required qualifications, if any.
    pub qualifications: Option<String>,
    /// Job responsibilities, if any.
    pub responsibilities: Option<String>,
    /// Notes from job review, if any.
    pub review_notes: Option<String>,
    /// Salary amount, if specified.
    pub salary: Option<i64>,
    /// Salary normalized to USD per year, if available.
    pub salary_usd_year: Option<i64>,
    /// Currency of the salary, if specified.
    pub salary_currency: Option<String>,
    /// Minimum salary, if specified.
    pub salary_min: Option<i64>,
    /// Minimum salary normalized to USD per year, if available.
    pub salary_min_usd_year: Option<i64>,
    /// Maximum salary, if specified.
    pub salary_max: Option<i64>,
    /// Maximum salary normalized to USD per year, if available.
    pub salary_max_usd_year: Option<i64>,
    /// Salary period (e.g., year, month, week, day, hour), if specified.
    pub salary_period: Option<String>,
    /// Seniority level for the job, if specified.
    pub seniority: Option<Seniority>,
    /// List of required or desired skills, if any.
    pub skills: Option<Vec<String>>,
    /// End of timezone range, if specified.
    pub tz_end: Option<String>,
    /// Start of timezone range, if specified.
    pub tz_start: Option<String>,
    /// Timestamp when the job was last updated, if available.
    pub updated_at: Option<DateTime<Utc>>,
    /// Upstream commitment level, if specified.
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
