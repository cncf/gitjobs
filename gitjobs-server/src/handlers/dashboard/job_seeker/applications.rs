//! This module defines the HTTP handlers for the applications page.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession, db::DynDB, handlers::error::HandlerError,
    templates::dashboard::job_seeker::applications::ApplicationsPage,
};

// Pages handlers.

/// Handler that returns the applications list page.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare template
    let applications = db.list_job_seeker_applications(&user.user_id).await?;
    let template = ApplicationsPage { applications };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Handler that cancels an application.
#[instrument(skip_all, err)]
pub(crate) async fn cancel(
    auth_session: AuthSession,
    Path(application_id): Path<Uuid>,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok((StatusCode::FORBIDDEN).into_response());
    };

    // Cancel application
    db.cancel_application(&application_id, &user.user_id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [(
            "HX-Location",
            r#"{"path":"/dashboard/job-seeker?tab=applications", "target":"body"}"#,
        )],
    )
        .into_response())
}
