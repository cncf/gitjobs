//! This module defines some templates and types used in the employer dashboard
//! jobs page.

use chrono::{DateTime, Utc};
use rinja::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    filters,
    helpers::{DATE_FORMAT, build_dashboard_image_url, format_location, normalize},
    misc::{Location, Project},
};

use super::employers::Employer;

/// Add job page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/jobs/add.html")]
pub(crate) struct AddPage {
    pub benefits: Vec<String>,
    pub skills: Vec<String>,
}

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
    pub benefits: Vec<String>,
    pub job: Job,
    pub skills: Vec<String>,
}

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
    pub salary_currency: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_period: Option<String>,
    pub skills: Option<Vec<String>>,
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum JobStatus {
    Archived,
    #[default]
    Draft,
    Published,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Archived => write!(f, "archived"),
            JobStatus::Draft => write!(f, "draft"),
            JobStatus::Published => write!(f, "published"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "archived" => Ok(JobStatus::Archived),
            "draft" => Ok(JobStatus::Draft),
            "published" => Ok(JobStatus::Published),
            _ => Err("invalid job status".to_string()),
        }
    }
}

/// Job kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum JobKind {
    Contractor,
    Internship,
    FullTime,
    PartTime,
}

impl std::fmt::Display for JobKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobKind::Contractor => write!(f, "contractor"),
            JobKind::Internship => write!(f, "internship"),
            JobKind::FullTime => write!(f, "full-time"),
            JobKind::PartTime => write!(f, "part-time"),
        }
    }
}

impl std::str::FromStr for JobKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "contractor" => Ok(JobKind::Contractor),
            "internship" => Ok(JobKind::Internship),
            "full-time" => Ok(JobKind::FullTime),
            "part-time" => Ok(JobKind::PartTime),
            _ => Err("invalid job type".to_string()),
        }
    }
}

/// Salary kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SalaryKind {
    Fixed,
    Range,
}

impl std::fmt::Display for SalaryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SalaryKind::Fixed => write!(f, "fixed"),
            SalaryKind::Range => write!(f, "range"),
        }
    }
}

/// Job workplace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Workplace {
    Hybrid,
    OnSite,
    Remote,
}

impl std::fmt::Display for Workplace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Workplace::Hybrid => write!(f, "hybrid"),
            Workplace::OnSite => write!(f, "on-site"),
            Workplace::Remote => write!(f, "remote"),
        }
    }
}

impl std::str::FromStr for Workplace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hybrid" => Ok(Workplace::Hybrid),
            "on-site" => Ok(Workplace::OnSite),
            "remote" => Ok(Workplace::Remote),
            _ => Err("invalid workplace".to_string()),
        }
    }
}

/// Job board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobBoard {
    pub benefits: Vec<String>,
    pub skills: Vec<String>,
}
