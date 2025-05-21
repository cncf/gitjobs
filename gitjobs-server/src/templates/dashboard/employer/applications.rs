//! Templates and types for the employer dashboard applications page.

use anyhow::Result;
use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    dashboard::employer::jobs::{JobSummary, Workplace},
    helpers::{DATE_FORMAT, build_dashboard_image_url},
    misc::Location,
    pagination::{NavigationLinks, Pagination},
};

// Pages templates.

/// Applications page template for employer dashboard.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/applications/list.html")]
pub(crate) struct ApplicationsPage {
    /// List of job applications.
    pub applications: Vec<Application>,
    /// Filters applied to the applications list.
    pub filters: Filters,
    /// Available filter options for the page.
    pub filters_options: FiltersOptions,
    /// Navigation links for pagination.
    pub navigation_links: NavigationLinks,
}

impl ApplicationsPage {
    /// Get selected job.
    pub(crate) fn selected_job(&self) -> Option<&JobSummary> {
        if let Some(job_id) = self.filters.job_id {
            return self.filters_options.jobs.iter().find(|j| j.job_id == job_id);
        }
        None
    }
}

// Types.

/// Application information for employer dashboard listings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Application {
    /// Unique identifier for the application.
    application_id: Uuid,
    /// Name of the applicant.
    name: String,
    /// Timestamp when the application was submitted.
    applied_at: DateTime<Utc>,
    /// Unique identifier for the job.
    job_id: Uuid,
    /// Title of the job applied for.
    job_title: String,
    /// Unique identifier for the job seeker profile.
    job_seeker_profile_id: Uuid,
    /// Workplace type for the job.
    job_workplace: Workplace,

    /// Location of the job, if specified.
    job_location: Option<Location>,
    /// Last position held by the applicant, if any.
    last_position: Option<String>,
    /// Photo identifier for the applicant, if available.
    photo_id: Option<Uuid>,
}

/// Filters used to search for applications.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Filters {
    /// Filter by job unique identifier.
    pub job_id: Option<Uuid>,
    /// Limit the number of results.
    pub limit: Option<usize>,
    /// Offset for pagination.
    pub offset: Option<usize>,
}

impl Filters {
    /// Convert the filters to a raw query string.
    #[allow(dead_code)]
    fn to_raw_query(&self) -> Result<String> {
        serde_qs::to_string(self).map_err(Into::into)
    }
}

impl Pagination for Filters {
    fn get_base_hx_url(&self) -> String {
        "/dashboard/employer/applications/list".to_string()
    }

    fn get_base_url(&self) -> String {
        "/dashboard/employer?tab=applications".to_string()
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

/// Filter options used in the applications page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    /// List of job summaries for filter selection.
    pub jobs: Vec<JobSummary>,
}
