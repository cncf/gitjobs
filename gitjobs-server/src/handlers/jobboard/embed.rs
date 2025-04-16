//! This module defines the HTTP handlers for the embed page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::Duration;
use serde_qs::axum::QsQuery;
use tracing::instrument;

use crate::{
    config::HttpServerConfig,
    db::{DynDB, jobboard::JobsSearchOutput},
    handlers::{error::HandlerError, prepare_headers},
    templates::jobboard::{embed::Page, jobs::Filters},
};

/// Handler that returns the embed page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get jobs that match the query
    let JobsSearchOutput { jobs, total: _ } = db.search_jobs(&filters).await?;

    // Prepare template
    let template = Page {
        base_url: cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url).to_string(),
        jobs,
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::minutes(10), &[])?;

    Ok((headers, Html(template.render()?)))
}
