//! This module defines the HTTP handlers for the jobs page.

use anyhow::Result;
use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::{DynDB, jobboard::JobsSearchOutput},
    handlers::{error::HandlerError, extractors::JobBoardId},
    templates::{
        PageId,
        jobboard::jobs::{ExploreSection, Filters, Page, ResultsSection},
    },
};

// Pages and sections handlers.

/// Handler that returns the jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and jobs that match the query
    let filters = Filters::new(&raw_query.unwrap_or_default())?;
    let (filters_options, JobsSearchOutput { jobs, total }) = tokio::try_join!(
        db.get_jobs_filters_options(),
        db.search_jobs(&job_board_id, &filters)
    )?;

    // Prepare template
    let offset = filters.offset;
    let template = Page {
        explore_section: ExploreSection {
            filters,
            filters_options,
            results_section: ResultsSection { jobs, total, offset },
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
    RawQuery(raw_query): RawQuery,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and jobs that match the query
    let filters = Filters::new(&raw_query.unwrap_or_default())?;
    let (filters_options, JobsSearchOutput { jobs, total }) = tokio::try_join!(
        db.get_jobs_filters_options(),
        db.search_jobs(&job_board_id, &filters)
    )?;

    // Prepare template
    let offset = filters.offset;
    let template = ExploreSection {
        filters,
        filters_options,
        results_section: ResultsSection { jobs, total, offset },
    };

    Ok(Html(template.render()?))
}

/// Handler that returns the results section.
#[instrument(skip_all, err)]
pub(crate) async fn results_section(
    State(db): State<DynDB>,
    RawQuery(raw_query): RawQuery,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    // Get jobs that match the query
    let filters = Filters::new(&raw_query.unwrap_or_default())?;
    let JobsSearchOutput { jobs, total } = db.search_jobs(&job_board_id, &filters).await?;

    // Prepare template
    let offset = filters.offset;
    let template = ResultsSection { jobs, total, offset };

    Ok(Html(template.render()?))
}
