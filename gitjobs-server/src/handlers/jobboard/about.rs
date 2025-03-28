//! This module defines the HTTP handlers for the about page.

use anyhow::{Result, anyhow};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use cached::proc_macro::cached;
use tracing::instrument;

use crate::{db::DynDB, handlers::error::HandlerError, templates::jobboard::about::Page};

/// Handler that returns the about page.
#[instrument(skip_all, err)]
pub(crate) async fn page(State(_db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let template = Page {
        content: prepare_content()?,
    };

    Ok(Html(template.render()?))
}

/// Prepare about page content.
#[cached(
    key = "&str",
    convert = r#"{ "about_content" }"#,
    sync_writes = "by_key",
    result = true
)]
pub(crate) fn prepare_content() -> Result<String> {
    let md = include_str!("../../../../docs/about.md");
    let options = markdown::Options::gfm();
    let html = markdown::to_html_with_options(md, &options).map_err(|e| anyhow!(e))?;
    Ok(html)
}
