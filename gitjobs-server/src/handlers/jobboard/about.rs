//! HTTP handlers for the about page.

use anyhow::{Result, anyhow};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use cached::proc_macro::cached;
use chrono::Duration;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    handlers::{auth::AUTH_PROVIDER_KEY, error::HandlerError, prepare_headers},
    templates::{PageId, jobboard::about::Page},
};

/// Handler that returns the about page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    session: Session,
    State(cfg): State<HttpServerConfig>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = Page {
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        content: prepare_content()?,
        page_id: PageId::About,
        user: auth_session.into(),
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)))
}

/// Prepares and caches the about page content as HTML from Markdown source.
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
