//! This module defines the HTTP handlers for the jobs moderation dashboard pages.

use anyhow::Result;
use askama::Template;
use axum::{
    Form,
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    db::DynDB,
    handlers::error::HandlerError,
    templates::{
        dashboard::{
            employer::{self, jobs::JobStatus},
            moderator::jobs,
        },
        helpers::option_is_none_or_default,
        notifications::JobPublished,
    },
};

// Pages handlers.

/// Returns the page listing all live (published) jobs for moderation.
#[instrument(skip_all, err)]
pub(crate) async fn live_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_jobs_for_moderation(JobStatus::Published).await?;
    let template = jobs::LivePage { jobs };

    Ok(Html(template.render()?))
}

/// Returns the page listing all jobs pending approval for moderation.
#[instrument(skip_all, err)]
pub(crate) async fn pending_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_jobs_for_moderation(JobStatus::PendingApproval).await?;
    let template = jobs::PendingPage { jobs };

    Ok(Html(template.render()?))
}

/// Returns the preview page for a specific job and its employer.
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

/// Approves a job as a moderator and triggers a table refresh in the UI.
#[instrument(skip_all, err)]
pub(crate) async fn approve(
    auth_session: AuthSession,
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Approve job
    let previous_first_published_at = db.approve_job(&job_id, &user.user_id).await?;

    // Post a Slack notification the first time a job is published
    if previous_first_published_at.is_none() {
        if let Some(webhook_url) = &cfg.slack_webhook_url {
            if let Some(job) = db.get_job_jobboard(&job_id).await? {
                let template = JobPublished { job };
                let payload = json!({
                    "text": template.render()?,
                });
                if let Err(err) = reqwest::Client::new().post(webhook_url).json(&payload).send().await {
                    warn!("error posting slack notification: {}", err);
                }
            }
        }
    }

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-moderator-table")],
    )
        .into_response())
}

/// Rejects a job as a moderator, optionally including review notes, and triggers a table
/// refresh.
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

/// Input data for rejecting a job, including optional review notes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RejectInput {
    /// Optional review notes provided by the moderator when rejecting a job.
    #[serde(skip_serializing_if = "option_is_none_or_default")]
    pub review_notes: Option<String>,
}
