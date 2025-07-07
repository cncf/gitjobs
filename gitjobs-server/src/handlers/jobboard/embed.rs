//! HTTP handlers for job board embed endpoints, including jobs and job card embeds.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use chrono::Duration;
use serde_qs::axum::QsQuery;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::HttpServerConfig,
    db::{DynDB, jobboard::JobsSearchOutput},
    handlers::{error::HandlerError, prepare_headers},
    templates::jobboard::{
        embed::{JobCard, JobsPage},
        jobs::Filters,
    },
};

/// Returns the jobs embed page for external integration.
#[instrument(skip_all, err)]
pub(crate) async fn jobs_page(
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get jobs that match the query
    let JobsSearchOutput { jobs, total: _ } = db.search_jobs(&filters).await?;

    // Prepare template
    let template = JobsPage {
        base_url: cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url).to_string(),
        jobs,
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::minutes(10), &[])?;

    Ok((headers, Html(template.render()?)))
}

/// Returns the job card embed as an SVG image for sharing or embedding.
#[instrument(skip_all, err)]
pub(crate) async fn job_card(
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = JobCard {
        base_url: cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url).to_string(),
        job: db.get_job_jobboard(&job_id).await?,
    };

    // Prepare response headers
    let extra_headers = [("content-type", "image/svg+xml")];
    let headers = prepare_headers(Duration::minutes(10), &extra_headers)?;

    Ok((headers, template.render()?))
}
