//! This module defines the HTTP handlers for the job board home page.

use anyhow::Result;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    db::DynDB,
    handlers::{error::HandlerError, extractors::JobBoardId},
    templates::{jobboard::home::Page, CurrentPage},
};

/// Handler that returns the home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    State(_db): State<DynDB>,
    JobBoardId(_job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    Ok(Html(
        Page {
            current_page: CurrentPage::JobBoard,
        }
        .render()?,
    ))
}
