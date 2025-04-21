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
    templates::{
        dashboard::{
            employer::{self, jobs::JobStatus},
            moderator::jobs,
        },
        helpers::option_is_none_or_default,
    },
};

// Pages handlers.

/// Handler that returns the live jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn live_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_jobs_for_moderation(JobStatus::Published).await?;
    let template = jobs::LivePage { jobs };

    Ok(Html(template.render()?))
}

/// Handler that returns the pending jobs page.
#[instrument(skip_all, err)]
pub(crate) async fn pending_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_jobs_for_moderation(JobStatus::PendingApproval).await?;
    let template = jobs::PendingPage { jobs };

    Ok(Html(template.render()?))
}

/// Handler that returns the job preview page.
#[instrument(skip_all, err)]
pub(crate) async fn preview_page(
    State(db): State<DynDB>,
    Path((employer_id, job_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HandlerError> {
    let (employer, job) = tokio::try_join!(db.get_employer(&employer_id), db.get_job_dashboard(&job_id))?;
    let template = employer::jobs::PreviewPage { employer, job };

    Ok(Html(template.render()?).into_response())
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
        [("HX-Trigger", "refresh-moderator-table")],
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
        [("HX-Trigger", "refresh-moderator-table")],
    )
        .into_response())
}

// Types.

/// Reject information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RejectInput {
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub review_notes: Option<String>,
}
