//! This module defines the HTTP handlers for previewing, updating, and saving job
//! seeker profiles.

use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    templates::dashboard::job_seeker::profile::{self, JobSeekerProfile},
};

// Pages handlers.

/// Handler that returns the page to preview a profile.
#[instrument(skip_all, err)]
pub(crate) async fn preview_page(
    State(serde_qs_de): State<serde_qs::Config>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get profile information from body
    let mut profile: JobSeekerProfile = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    profile.normalize();

    // Prepare template
    let template = profile::PreviewPage { profile };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the page to update a profile.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare template
    let profile = db.get_job_seeker_profile(&user.user_id).await?;
    let template = profile::UpdatePage { profile };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Handler that updates a job seeker's profile in the database.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    auth_session: AuthSession,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Get profile information from body
    let mut profile: JobSeekerProfile = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    profile.normalize();

    // Update profile in database
    db.update_job_seeker_profile(&user.user_id, &profile).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
