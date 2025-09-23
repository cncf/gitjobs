//! HTTP handlers for the stats page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::Duration;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    config::HttpServerConfig,
    db::DynDB,
    handlers::{auth::AUTH_PROVIDER_KEY, error::HandlerError, prepare_headers},
    templates::{PageId, auth::User, jobboard::stats::Page},
};

/// Handler that returns the stats page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    session: Session,
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get stats information from the database
    let stats = db.get_stats().await?;

    // Prepare template
    let template = Page {
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        page_id: PageId::Stats,
        stats,
        user: User::default(),
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)))
}
