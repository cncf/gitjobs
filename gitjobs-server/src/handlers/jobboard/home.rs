//! This module defines the HTTP handlers for the job board home page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    templates::{PageId, jobboard::home::Page},
};

/// Handler that returns the home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(_db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = Page {
        logged_in: auth_session.user.is_some(),
        page_id: PageId::JobBoard,
        name: auth_session.user.as_ref().map(|u| u.name.clone()),
        username: auth_session.user.as_ref().map(|u| u.username.clone()),
    };

    Ok(Html(template.render()?))
}
