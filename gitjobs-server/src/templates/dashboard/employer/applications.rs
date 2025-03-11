//! This module defines some templates and types used in the applications page.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rinja::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::trace;
use uuid::Uuid;

use crate::templates::misc::Location;

use super::jobs::JobSummary;

/// Applications page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/applications/list.html")]
pub(crate) struct ApplicationsPage {
    pub applications: Vec<Application>,
    pub filters: Filters,
    pub filters_options: FiltersOptions,
}

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
    /// Create a new `Filters` instance from the raw query string provided.
    pub(crate) fn new(serde_qs_de: &serde_qs::Config, raw_query: &str) -> Result<Self> {
        let filters: Filters = serde_qs_de.deserialize_str(raw_query)?;

        trace!("{:?}", filters);
        Ok(filters)
    }

    /// Convert the filters to a raw query string.
    #[allow(dead_code)]
    fn to_raw_query(&self) -> Result<String> {
        serde_qs::to_string(self).map_err(Into::into)
    }
}

/// Filters options used in the applications page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    pub jobs: Vec<JobSummary>,
}
