//! This module defines the HTTP handlers.

use anyhow::Result;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use chrono::Duration;
use reqwest::header::CACHE_CONTROL;

use crate::{
    db::{DynDB, jobboard::JobsSearchOutput},
    event_tracker::{DynEventTracker, Event},
    templates::jobboard::jobs::Filters,
};

/// Authentication-related HTTP handlers.
pub(crate) mod auth;
/// Dashboard-related HTTP handlers.
pub(crate) mod dashboard;
/// Error handling utilities for HTTP handlers.
pub(crate) mod error;
/// Custom extractors for HTTP handlers.
pub(crate) mod extractors;
/// Image-related HTTP handlers.
pub(crate) mod img;
/// Job board HTTP handlers.
pub(crate) mod jobboard;
/// Miscellaneous HTTP handlers.
pub(crate) mod misc;

/// Helper function to prepare headers for HTTP responses, including cache control and
/// additional custom headers.
#[allow(unused_variables)]
pub(crate) fn prepare_headers(cache_duration: Duration, extra_headers: &[(&str, &str)]) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    // Set cache control header
    #[cfg(debug_assertions)]
    let duration_secs = 0; // Disable caching in debug mode
    #[cfg(not(debug_assertions))]
    let duration_secs = cache_duration.num_seconds();
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::try_from(format!("max-age={duration_secs}"))?,
    );

    // Set extra headers
    for (key, value) in extra_headers {
        headers.insert(HeaderName::try_from(*key)?, HeaderValue::try_from(*value)?);
    }

    Ok(headers)
}

/// Searches for jobs and tracks search appearances.
pub(crate) async fn search_jobs(
    db: &DynDB,
    event_tracker: &DynEventTracker,
    filters: &Filters,
) -> Result<JobsSearchOutput> {
    // Perform the search
    let output = db.search_jobs(filters).await?;

    // Track search appearances if there are results
    if !output.jobs.is_empty() {
        let job_ids = output.jobs.iter().map(|j| j.job_id).collect();
        event_tracker.track(Event::SearchAppearances { job_ids }).await?;
    }

    Ok(output)
}
