//! Templates and types for the job seeker dashboard profile page.

use askama::Template;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    filters,
    helpers::{DATE_FORMAT_2, build_dashboard_image_url, normalize},
    misc::Location,
};

// Pages templates.

/// Template for the profile preview page in the job seeker dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/job_seeker/profile/preview.html")]
pub(crate) struct PreviewPage {
    /// Job seeker profile data to preview.
    pub profile: JobSeekerProfile,
}

/// Template for the update profile page in the job seeker dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/job_seeker/profile/update.html")]
pub(crate) struct UpdatePage {
    /// Job seeker profile data to update.
    pub profile: Option<JobSeekerProfile>,
}

// Types.

/// Represents a job seeker's profile and related information.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct JobSeekerProfile {
    /// Email address of the job seeker.
    pub email: String,
    /// Full name of the job seeker.
    pub name: String,
    /// Whether the profile is public.
    pub public: bool,
    /// Short summary or bio.
    pub summary: String,

    /// Bluesky profile URL.
    pub bluesky_url: Option<String>,
    /// List of certifications.
    pub certifications: Option<Vec<Certification>>,
    /// List of education entries.
    pub education: Option<Vec<Education>>,
    /// List of work experiences.
    pub experience: Option<Vec<Experience>>,
    /// Facebook profile URL.
    pub facebook_url: Option<String>,
    /// GitHub profile URL.
    pub github_url: Option<String>,
    /// LinkedIn profile URL.
    pub linkedin_url: Option<String>,
    /// Location of the job seeker.
    pub location: Option<Location>,
    /// Willingness to relocate.
    pub open_to_relocation: Option<bool>,
    /// Willingness to work remotely.
    pub open_to_remote: Option<bool>,
    /// Phone number.
    pub phone: Option<String>,
    /// Photo identifier.
    pub photo_id: Option<Uuid>,
    /// List of projects.
    pub projects: Option<Vec<Project>>,
    /// List of skills.
    pub skills: Option<Vec<String>>,
    /// Twitter profile URL.
    pub twitter_url: Option<String>,
    /// Personal website URL.
    pub website_url: Option<String>,
}

impl JobSeekerProfile {
    /// Normalize some fields in the job seeker profile.
    pub(crate) fn normalize(&mut self) {
        // Skills
        if let Some(skills) = &mut self.skills {
            for skill in skills.iter_mut() {
                *skill = normalize(skill);
            }
        }
    }
}

/// Certification details for a job seeker profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Certification {
    /// Description of the certification.
    pub description: String,
    /// End date of the certification.
    pub end_date: NaiveDate,
    /// Provider of the certification.
    pub provider: String,
    /// Start date of the certification.
    pub start_date: NaiveDate,
    /// Title of the certification.
    pub title: String,
}

/// Education details for a job seeker profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Education {
    /// Description of the education.
    pub description: String,
    /// Name of the educational institution.
    pub educational_institution: String,
    /// End date of the education.
    pub end_date: NaiveDate,
    /// Start date of the education.
    pub start_date: NaiveDate,
    /// Title or degree obtained.
    pub title: String,
}

/// Work experience details for a job seeker profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Experience {
    /// Name of the company.
    pub company: String,
    /// Description of the work experience.
    pub description: String,
    /// Start date of the experience.
    pub start_date: NaiveDate,
    /// Job title.
    pub title: String,

    /// Optional end date of the experience.
    pub end_date: Option<NaiveDate>,
}

/// Project details for a job seeker profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Project {
    /// Description of the project.
    pub description: String,
    /// Title of the project.
    pub title: String,
    /// Main URL for the project.
    pub url: String,

    /// Optional source code URL for the project.
    pub source_url: Option<String>,
}
