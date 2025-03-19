//! This module defines the HTTP handlers for the about page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use tracing::instrument;

use crate::{db::DynDB, handlers::error::HandlerError, templates::jobboard::about::Page};

/// Handler that returns the about page.
#[instrument(skip_all, err)]
pub(crate) async fn page(State(_db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    Ok(Html(Page {}.render()?))
}
