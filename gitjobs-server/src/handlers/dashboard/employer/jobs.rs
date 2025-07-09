//! HTTP handlers for employer job management pages and actions.
//
// This module provides handlers for adding, listing, previewing, updating, archiving,
// deleting, and publishing jobs for employers in the dashboard. It also renders the
// corresponding Askama templates for each page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::{Duration, Utc};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{error::HandlerError, extractors::SelectedEmployerIdRequired, prepare_headers},
    templates::dashboard::employer::jobs::{self, Job, JobStatus},
};

// Pages handlers.

/// Renders the page to add a new job for an employer.
#[instrument(skip_all, err)]
pub(crate) async fn add_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let (certifications, foundations) = tokio::try_join!(
        db.list_certifications(),
        db.list_foundations()
    )?;
    let template = jobs::AddPage { certifications, foundations };

    Ok(Html(template.render()?))
}

/// Renders the jobs list page for the selected employer.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_employer_jobs(&employer_id).await?;
    let template = jobs::ListPage { jobs };

    Ok(Html(template.render()?))
}

/// Handler that returns the job preview page (job provided in body).
#[instrument(skip_all, err)]
pub(crate) async fn preview_page_w_job(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize().await;
    job.published_at = Some(Utc::now());
    job.updated_at = Some(Utc::now());

    // Prepare template
    let employer = db.get_employer(&employer_id).await?;
    let template = jobs::PreviewPage { employer, job };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the job preview page (job not provided in body).
#[instrument(skip_all, err)]
pub(crate) async fn preview_page_wo_job(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let (employer, job) = tokio::try_join!(db.get_employer(&employer_id), db.get_job_dashboard(&job_id))?;
    let template = jobs::PreviewPage { employer, job };

    Ok(Html(template.render()?).into_response())
}

/// Renders the page to update an existing job.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let (certifications, foundations, job) = tokio::try_join!(
        db.list_certifications(),
        db.list_foundations(),
        db.get_job_dashboard(&job_id)
    )?;
    let template = jobs::UpdatePage { certifications, foundations, job };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Adds a new job for the selected employer.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize().await;

    // Make sure the status provided is valid
    if job.status != JobStatus::Draft && job.status != JobStatus::PendingApproval {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, "invalid status").into_response());
    }

    // Add job to database
    db.add_job(&employer_id, &job).await?;

    Ok((StatusCode::CREATED, [("HX-Trigger", "refresh-jobs-table")]).into_response())
}

/// Archives a job, making it inactive but not deleting it.
#[instrument(skip_all, err)]
pub(crate) async fn archive(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.archive_job(&job_id).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]))
}

/// Permanently deletes a job from the database.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_job(&job_id).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]))
}

/// Publishes a job. It'll be visible to users once it's approved.
#[instrument(skip_all, err)]
pub(crate) async fn publish(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.publish_job(&job_id).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]))
}

/// Returns statistics for a specific job.
#[instrument(skip_all, err)]
pub(crate) async fn stats(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get stats information from the database
    let stats = db.get_job_stats(&job_id).await?;

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Json(stats)))
}

/// Updates an existing job with new data.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    Path(job_id): Path<Uuid>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize().await;

    // Make sure the status provided is valid
    if job.status != JobStatus::Archived
        && job.status != JobStatus::Draft
        && job.status != JobStatus::PendingApproval
    {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, "invalid status").into_response());
    }

    // Update job in database
    db.update_job(&job_id, &job).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]).into_response())
}
