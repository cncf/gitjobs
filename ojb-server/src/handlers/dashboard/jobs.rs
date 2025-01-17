//! This module defines the HTTP handlers for the jobs page.

use anyhow::Result;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::{debug, instrument};

use crate::{
    db::DynDB,
    handlers::{error::HandlerError, extractors::JobBoardId},
    templates::dashboard::jobs::Page,
};

/// Handler that returns the jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    State(_db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    debug!("job_board_id: {}", job_board_id);

    Ok(Html(Page {}.render()?))
}
