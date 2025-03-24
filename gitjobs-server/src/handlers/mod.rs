//! This module defines the HTTP handlers.

use anyhow::Result;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use chrono::Duration;
use reqwest::header::CACHE_CONTROL;

pub(crate) mod auth;
pub(crate) mod dashboard;
pub(crate) mod error;
pub(crate) mod extractors;
pub(crate) mod img;
pub(crate) mod jobboard;
pub(crate) mod misc;

/// Helper function to prepare headers for HTTP responses.
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
