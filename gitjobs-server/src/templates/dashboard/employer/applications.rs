//! This module defines some templates and types used in the applications page.

use anyhow::Result;
use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::templates::{
    dashboard::employer::jobs::JobSummary,
    helpers::{DATE_FORMAT, build_dashboard_image_url},
    misc::Location,
    pagination::{NavigationLinks, Pagination},
};

// Pages templates.

/// Applications page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/applications/list.html")]
pub(crate) struct ApplicationsPage {
    pub applications: Vec<Application>,
    pub filters: Filters,
    pub filters_options: FiltersOptions,
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

/// Application information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Application {
    application_id: Uuid,
    name: String,
    applied_at: DateTime<Utc>,
    job_id: Uuid,
    job_title: String,
    job_seeker_profile_id: Uuid,

    job_location: Option<Location>,
    last_position: Option<String>,
    photo_id: Option<Uuid>,
}

/// Filters used to search for applications.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Filters {
    pub job_id: Option<Uuid>,
    pub limit: Option<usize>,
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

/// Filters options used in the applications page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    pub jobs: Vec<JobSummary>,
}
