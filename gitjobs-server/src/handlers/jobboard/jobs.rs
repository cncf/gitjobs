//! This module defines the HTTP handlers for the jobs pages.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use chrono::Duration;
use reqwest::StatusCode;
use serde_qs::axum::QsQuery;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::{DynDB, jobboard::JobsSearchOutput},
    handlers::{error::HandlerError, prepare_headers},
    templates::{
        PageId,
        jobboard::jobs::{ExploreSection, Filters, JobSection, JobsPage, ResultsSection},
        pagination::{NavigationLinks, build_url},
    },
};

// Pages and sections handlers.

/// Handler that returns the jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn jobs_page(
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and jobs that match the query
    let (filters_options, JobsSearchOutput { jobs, total }) =
        tokio::try_join!(db.get_jobs_filters_options(), db.search_jobs(&filters))?;

    // Prepare template
    let template = JobsPage {
        explore_section: ExploreSection {
            filters: filters.clone(),
            filters_options,
            results_section: ResultsSection {
                jobs,
                navigation_links: NavigationLinks::from_filters(&filters, total)?,
                total,
                offset: filters.offset,
            },
        },
        page_id: PageId::JobBoard,
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::minutes(10), &[])?;

    Ok((headers, Html(template.render()?)))
}

/// Handler that returns the results section.
#[instrument(skip_all, err)]
pub(crate) async fn results_section(
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get jobs that match the query
    let JobsSearchOutput { jobs, total } = db.search_jobs(&filters).await?;

    // Prepare template
    let template = ResultsSection {
        navigation_links: NavigationLinks::from_filters(&filters, total)?,
        jobs,
        total,
        offset: filters.offset,
    };

    // Prepare response headers
    let url = build_url("/", &filters)?;
    let extra_headers = [("HX-Replace-Url", url.as_str())];
    let headers = prepare_headers(Duration::minutes(10), &extra_headers)?;

    Ok((headers, Html(template.render()?)))
}

/// Handler that returns the job details section.
#[instrument(skip_all, err)]
pub(crate) async fn job_section(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information
    let Some(job) = db.get_job_jobboard(&job_id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // Prepare template
    let template = JobSection { job };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)).into_response())
}

// Actions handlers.

/// Handler that allows users to apply to a job.
#[instrument(skip_all, err)]
pub(crate) async fn apply(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN);
    };

    // Create job application entry in the database
    db.apply_to_job(&job_id, &user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
