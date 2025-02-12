//! This module defines the HTTP handlers for the profile page.

// Pages handlers.

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use rinja::Template;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    templates::dashboard::job_seeker::profile::{self},
};

/// Handler that returns the page to preview a profile.
#[instrument(skip_all, err)]
pub(crate) async fn preview_page(body: String) -> Result<impl IntoResponse, HandlerError> {
    let profile = match serde_qs::from_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    let template = profile::PreviewPage { profile };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the page to update a profile.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };
    let profile = db.get_job_seeker_profile(&user.user_id).await?;
    let template = profile::UpdatePage { profile };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Handler that updates a profile.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    auth_session: AuthSession,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Get profile information from body
    let profile = match serde_qs::from_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };

    // Update profile in database
    db.update_job_seeker_profile(&user.user_id, &profile).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
