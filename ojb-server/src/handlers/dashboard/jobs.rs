//! This module defines the HTTP handlers for the jobs page.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use rinja::Template;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{EmployerId, JobBoardId},
    },
    templates::dashboard::jobs,
};

// Pages handlers.

/// Handler that returns the page to add a new job.
#[instrument(skip_all, err)]
pub(crate) async fn add_page(
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    let job_board = db.get_job_board(job_board_id).await?;
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
    EmployerId(employer_id): EmployerId,
) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_employer_jobs(employer_id).await?;
    let template = jobs::ListPage { jobs };

    Ok((StatusCode::OK, Html(template.render()?)))
}

/// Handler that returns the page to update a job.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    JobBoardId(job_board_id): JobBoardId,
) -> Result<impl IntoResponse, HandlerError> {
    let (job_board, job_details) =
        tokio::try_join!(db.get_job_board(job_board_id), db.get_job_details(job_id),)?;

    let template = jobs::UpdatePage {
        benefits: job_board.benefits,
        job_details,
        skills: job_board.skills,
    };

    Ok(Html(template.render()?))
}

// Actions handlers.

/// Handler that adds a job.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    State(db): State<DynDB>,
    EmployerId(employer_id): EmployerId,
    Form(job_details): Form<jobs::JobDetails>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_job(employer_id, job_details).await?;

    Ok((StatusCode::CREATED, [("HX-Push-Url", "/dashboard/jobs/list")]))
}

/// Handler that archives a job.
#[instrument(skip_all, err)]
pub(crate) async fn archive(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.archive_job(job_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that deletes a job.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_job(job_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that publishes a job.
#[instrument(skip_all, err)]
pub(crate) async fn publish(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.publish_job(job_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that updates a job.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    Form(job_details): Form<jobs::JobDetails>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_job(job_id, job_details).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Push-Url", "/dashboard/jobs/list")]))
}
