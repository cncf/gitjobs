//! This module defines the HTTP handlers for the jobs pages.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde_qs::axum::QsQuery;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::{DynDB, jobboard::JobsSearchOutput},
    handlers::{error::HandlerError, extractors::JobBoardId},
    templates::{
        PageId,
        jobboard::jobs::{ExploreSection, Filters, JobPage, JobsPage, ResultsSection},
        pagination::{NavigationLinks, build_url},
    },
};

// Pages and sections handlers.

/// Handler that returns the jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn jobs_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and jobs that match the query
    let (filters_options, JobsSearchOutput { jobs, total }) = tokio::try_join!(
        db.get_jobs_filters_options(&job_board_id),
        db.search_jobs(&job_board_id, &filters)
    )?;

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
        logged_in: auth_session.user.is_some(),
        page_id: PageId::JobBoard,
        name: auth_session.user.as_ref().map(|u| u.name.clone()),
        username: auth_session.user.as_ref().map(|u| u.username.clone()),
    };

    Ok(Html(template.render()?))
}

/// Handler that returns the explore section.
#[instrument(skip_all, err)]
pub(crate) async fn explore_section(
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and jobs that match the query
    let (filters_options, JobsSearchOutput { jobs, total }) = tokio::try_join!(
        db.get_jobs_filters_options(&job_board_id),
        db.search_jobs(&job_board_id, &filters)
    )?;

    // Prepare template
    let template = ExploreSection {
        filters: filters.clone(),
        filters_options,
        results_section: ResultsSection {
            navigation_links: NavigationLinks::from_filters(&filters, total)?,
            jobs,
            total,
            offset: filters.offset,
        },
    };

    Ok(Html(template.render()?))
}

/// Handler that returns the results section.
#[instrument(skip_all, err)]
pub(crate) async fn results_section(
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    // Get jobs that match the query
    let JobsSearchOutput { jobs, total } = db.search_jobs(&job_board_id, &filters).await?;

    // Prepare template
    let template = ResultsSection {
        navigation_links: NavigationLinks::from_filters(&filters, total)?,
        jobs,
        total,
        offset: filters.offset,
    };

    // Prepare response headers
    let headers = [("HX-Push-Url", build_url("/jobs", &filters)?)];

    Ok((headers, Html(template.render()?)))
}

/// Handler that returns the job page.
#[instrument(skip_all, err)]
pub(crate) async fn job_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information
    let Some(job) = db.get_job_jobboard(&job_id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // Prepare template
    let template = JobPage {
        job,
        logged_in: auth_session.user.is_some(),
        page_id: PageId::JobBoard,
        name: auth_session.user.as_ref().map(|u| u.name.clone()),
        username: auth_session.user.as_ref().map(|u| u.username.clone()),
    };

    Ok(Html(template.render()?).into_response())
}
