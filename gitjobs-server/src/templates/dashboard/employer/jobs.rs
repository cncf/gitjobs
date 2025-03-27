//! This module defines some templates and types used in the employer dashboard
//! jobs page.

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
    misc::{Location, Project},
};

// Pages templates.

/// Add job page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/add.html")]
pub(crate) struct AddPage {}

/// Jobs list page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/list.html")]
pub(crate) struct ListPage {
    pub jobs: Vec<JobSummary>,
}

/// Job preview page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/preview.html")]
pub(crate) struct PreviewPage {
    pub employer: Employer,
    pub job: Job,
}

/// Update job page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/update.html")]
pub(crate) struct UpdatePage {
    pub job: Job,
}

// Types.

/// Job summary.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct JobSummary {
    pub job_id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub title: String,
    pub status: JobStatus,

    pub archived_at: Option<DateTime<Utc>>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Job details.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Job {
    pub description: String,
    pub status: JobStatus,
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
    pub salary_usd_year: Option<i64>,
    pub salary_currency: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_min_usd_year: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_max_usd_year: Option<i64>,
    pub salary_period: Option<String>,
    pub seniority: Option<Seniority>,
    pub skills: Option<Vec<String>>,
    pub tz_end: Option<String>,
    pub tz_start: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub upstream_commitment: Option<i32>,
}

impl Job {
    /// Normalize some fields.
    pub(crate) fn normalize(&mut self) {
        // Benefits
        if let Some(benefits) = &mut self.benefits {
            for benefit in benefits.iter_mut() {
                *benefit = normalize(benefit);
            }
        }

        // Salary (to USD yearly)
        let (currency, period) = (self.salary_currency.as_ref(), self.salary_period.as_ref());
        self.salary_usd_year = normalize_salary(self.salary, currency, period);
        self.salary_min_usd_year = normalize_salary(self.salary_min, currency, period);
        self.salary_max_usd_year = normalize_salary(self.salary_max, currency, period);

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

/// Job status.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum JobStatus {
    Archived,
    #[default]
    Draft,
    Published,
}

/// Job kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum JobKind {
    Contractor,
    Internship,
    FullTime,
    PartTime,
}

/// Salary kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum SalaryKind {
    Fixed,
    Range,
}

/// Job workplace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Workplace {
    Hybrid,
    OnSite,
    Remote,
}
