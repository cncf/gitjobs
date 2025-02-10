//! This module defines some templates and types used in the job seeker
//! dashboard profile page.

use chrono::NaiveDate;
use rinja::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Update profile page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/job_seeker/profile/update.html")]
pub(crate) struct UpdatePage {
    pub profile: Option<JobSeekerProfile>,
}

/// Job seeker profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobSeekerProfile {
    pub email: String,
    pub name: String,
    pub public: bool,
    pub summary: String,

    pub certifications: Option<Vec<Certification>>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub education: Option<Vec<Education>>,
    pub employments: Option<Vec<Employment>>,
    pub facebook_url: Option<String>,
    pub github_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub location_id: Option<Uuid>,
    pub open_to_relocation: Option<bool>,
    pub open_to_remote: Option<bool>,
    pub phone: Option<String>,
    pub photo_id: Option<Uuid>,
    pub photo_url: Option<String>,
    pub projects: Option<Vec<Project>>,
    pub skills: Option<Vec<String>>,
    pub state: Option<String>,
    pub twitter_url: Option<String>,
    pub website_url: Option<String>,
}

/// Certification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Certification {
    pub description: String,
    pub end_date: NaiveDate,
    pub provider: String,
    pub start_date: NaiveDate,
    pub title: String,
}

/// Education.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Education {
    pub description: String,
    pub educational_institution: String,
    pub end_date: NaiveDate,
    pub start_date: NaiveDate,
    pub title: String,
}

/// Employment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Employment {
    pub company: String,
    pub description: String,
    pub start_date: NaiveDate,
    pub title: String,

    pub end_date: Option<NaiveDate>,
}

/// Project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Project {
    pub description: String,
    pub title: String,
    pub url: String,

    pub source_url: Option<String>,
}
