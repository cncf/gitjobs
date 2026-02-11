//! Templates and types for the job seeker dashboard profile page.

use askama::Template;
use chrono::NaiveDate;
use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    templates::{
        filters,
        helpers::{DATE_FORMAT_2, build_dashboard_image_url, normalize},
        misc::Location,
    },
    validation::{
        MAX_LEN_BIO, MAX_LEN_DISPLAY_NAME, MAX_LEN_L, MAX_LEN_M, trimmed_non_empty, trimmed_non_empty_opt,
        trimmed_non_empty_tag_vec,
    },
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub(crate) struct JobSeekerProfile {
    /// Email address of the job seeker.
    #[garde(email, length(max = MAX_LEN_M))]
    pub email: String,
    /// Full name of the job seeker.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DISPLAY_NAME))]
    pub name: String,
    /// Whether the profile is public.
    #[garde(skip)]
    pub public: bool,
    /// Short summary or bio.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_BIO))]
    pub summary: String,

    /// Bluesky profile URL.
    #[garde(url, length(max = MAX_LEN_L))]
    pub bluesky_url: Option<String>,
    /// List of certifications.
    #[garde(skip)]
    pub certifications: Option<Vec<Certification>>,
    /// List of education entries.
    #[garde(skip)]
    pub education: Option<Vec<Education>>,
    /// List of work experiences.
    #[garde(skip)]
    pub experience: Option<Vec<Experience>>,
    /// Facebook profile URL.
    #[garde(url, length(max = MAX_LEN_L))]
    pub facebook_url: Option<String>,
    /// GitHub profile URL.
    #[garde(url, length(max = MAX_LEN_L))]
    pub github_url: Option<String>,
    /// `LinkedIn` profile URL.
    #[garde(url, length(max = MAX_LEN_L))]
    pub linkedin_url: Option<String>,
    /// Location of the job seeker.
    #[garde(skip)]
    pub location: Option<Location>,
    /// Willingness to relocate.
    #[garde(skip)]
    pub open_to_relocation: Option<bool>,
    /// Willingness to work remotely.
    #[garde(skip)]
    pub open_to_remote: Option<bool>,
    /// Phone number.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_M))]
    pub phone: Option<String>,
    /// Photo identifier.
    #[garde(skip)]
    pub photo_id: Option<Uuid>,
    /// List of projects.
    #[garde(skip)]
    pub projects: Option<Vec<Project>>,
    /// List of skills.
    #[garde(custom(trimmed_non_empty_tag_vec))]
    pub skills: Option<Vec<String>>,
    /// Twitter profile URL.
    #[garde(url, length(max = MAX_LEN_L))]
    pub twitter_url: Option<String>,
    /// Personal website URL.
    #[garde(url, length(max = MAX_LEN_L))]
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
