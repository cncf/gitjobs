//! This module defines the HTTP handlers for the jobs pages.

use askama::Template;
use axum::{
    Form,
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    templates::{dashboard::moderator::jobs, helpers::option_is_none_or_default},
};

// Pages handlers.

/// Handler that returns the pending jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn pending_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_moderation_pending_jobs().await?;
    let template = jobs::PendingPage { jobs };

    Ok(Html(template.render()?))
}

// Actions.

/// Handler that approves a job.
#[instrument(skip_all, err)]
pub(crate) async fn approve(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Approve job
    db.approve_job(&job_id, &user.user_id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-pending-jobs-table")],
    )
        .into_response())
}

/// Handler that rejects a job.
#[instrument(skip_all, err)]
pub(crate) async fn reject(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    Form(input): Form<RejectInput>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Reject job
    db.reject_job(&job_id, &user.user_id, input.review_notes.as_ref())
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-pending-jobs-table")],
    )
        .into_response())
}

// Types.

/// Reject information.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RejectInput {
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    review_notes: Option<String>,
}
