//! This module defines some templates and types used in the job seeker
//! dashboard profile page.

use chrono::NaiveDate;
use rinja::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    filters,
    helpers::{build_image_url, build_location, normalize, DATE_FORMAT_2},
};

/// Profile preview page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/job_seeker/profile/preview.html")]
pub(crate) struct PreviewPage {
    pub profile: JobSeekerProfile,
}

/// Update profile page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/job_seeker/profile/update.html")]
pub(crate) struct UpdatePage {
    pub profile: Option<JobSeekerProfile>,
    pub skills: Vec<String>,
}

/// Job seeker profile.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct JobSeekerProfile {
    pub email: String,
    pub name: String,
    pub public: bool,
    pub summary: String,

    pub certifications: Option<Vec<Certification>>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub education: Option<Vec<Education>>,
    pub experience: Option<Vec<Experience>>,
    pub facebook_url: Option<String>,
    pub github_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub location_id: Option<Uuid>,
    pub open_to_relocation: Option<bool>,
    pub open_to_remote: Option<bool>,
    pub phone: Option<String>,
    pub photo_id: Option<Uuid>,
    pub projects: Option<Vec<Project>>,
    pub skills: Option<Vec<String>>,
    pub state: Option<String>,
    pub twitter_url: Option<String>,
    pub website_url: Option<String>,
}

impl JobSeekerProfile {
    /// Normalize some fields.
    pub(crate) fn normalize(&mut self) {
        // Skills
        if let Some(skills) = &mut self.skills {
            for skill in skills.iter_mut() {
                *skill = normalize(skill);
            }
        }
    }
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

/// Experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Experience {
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
