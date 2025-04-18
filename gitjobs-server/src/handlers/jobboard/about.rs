//! This module defines the HTTP handlers for the about page.

use anyhow::{Result, anyhow};
use askama::Template;
use axum::response::{Html, IntoResponse};
use cached::proc_macro::cached;
use chrono::Duration;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    handlers::{error::HandlerError, prepare_headers},
    templates::{PageId, jobboard::about::Page},
};

/// Handler that returns the about page.
#[instrument(skip_all, err)]
pub(crate) async fn page(auth_session: AuthSession) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = Page {
        content: prepare_content()?,
        page_id: PageId::About,
        user: auth_session.into(),
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)))
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
