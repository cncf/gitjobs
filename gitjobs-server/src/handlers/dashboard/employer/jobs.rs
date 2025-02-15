//! This module defines the HTTP handlers for the jobs page.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use chrono::Utc;
use rinja::Template;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{JobBoardId, SelectedEmployerIdRequired},
    },
    templates::dashboard::employer::jobs::{self, Job},
};

// Pages handlers.

/// Handler that returns the page to add a new job.
#[instrument(skip_all, err)]
pub(crate) async fn add_page(
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    let job_board = db.get_job_board(&job_board_id).await?;
    let template = jobs::AddPage {
        benefits: job_board.benefits,
        skills: job_board.skills,
    };

    Ok(Html(template.render()?))
}

/// Handler that returns the jobs list page.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_employer_jobs(&employer_id).await?;
    let template = jobs::ListPage { jobs };

    Ok(Html(template.render()?))
}

/// Handler that returns the job preview page.
#[instrument(skip_all, err)]
pub(crate) async fn preview_page(
    State(db): State<DynDB>,
    State(form_de): State<serde_qs::Config>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match form_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize();
    job.published_at = Some(Utc::now());
    job.updated_at = Some(Utc::now());

    let employer = db.get_employer(&employer_id).await?;
    let template = jobs::PreviewPage { employer, job };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the page to update a job.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    let (job_board, job) = tokio::try_join!(db.get_job_board(&job_board_id), db.get_job(&job_id))?;
    let template = jobs::UpdatePage {
        benefits: job_board.benefits,
        job,
        skills: job_board.skills,
    };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Handler that adds a job.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    State(db): State<DynDB>,
    State(form_de): State<serde_qs::Config>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match form_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize();

    // Add job to database
    db.add_job(&employer_id, &job).await?;

    Ok(StatusCode::CREATED.into_response())
}

/// Handler that archives a job.
#[instrument(skip_all, err)]
pub(crate) async fn archive(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.archive_job(&job_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that deletes a job.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_job(&job_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that publishes a job.
#[instrument(skip_all, err)]
pub(crate) async fn publish(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.publish_job(&job_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that updates a job.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    State(form_de): State<serde_qs::Config>,
    Path(job_id): Path<Uuid>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match form_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize();

    // Update job in database
    db.update_job(&job_id, &job).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
