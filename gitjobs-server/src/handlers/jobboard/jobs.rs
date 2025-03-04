//! This module defines the HTTP handlers for the jobs page.

use anyhow::Result;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{error::HandlerError, extractors::JobBoardId},
    templates::{
        PageId,
        jobboard::jobs::{ExploreSection, Page, ResultsSection},
    },
};

// Pages and sections handlers.

/// Handler that returns the jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(_db): State<DynDB>,
    JobBoardId(_job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    let template = Page {
        logged_in: auth_session.user.is_some(),
        page_id: PageId::JobBoard,
        name: auth_session.user.as_ref().map(|u| u.name.clone()),
        username: auth_session.user.as_ref().map(|u| u.username.clone()),
    };

    Ok(Html(template.render()?))
}

/// Handler that returns the explore section.
#[instrument(skip_all, err)]
pub(crate) async fn explore_section() -> Result<impl IntoResponse, HandlerError> {
    let template = ExploreSection {};

    Ok(Html(template.render()?))
}

/// Handler that returns the results section.
#[instrument(skip_all, err)]
pub(crate) async fn results_section() -> Result<impl IntoResponse, HandlerError> {
    let template = ResultsSection {};

    Ok(Html(template.render()?))
}
