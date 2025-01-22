//! This module defines the HTTP handlers for the jobs page.

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use rinja::Template;
use tracing::instrument;

use crate::{
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{EmployerId, JobBoardId},
    },
    templates::dashboard::jobs,
};

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

/// Handler that adds a job to the job board.
#[instrument(skip_all, err)]
pub(crate) async fn add_job(
    State(db): State<DynDB>,
    EmployerId(employer_id): EmployerId,
    Form(job): Form<jobs::NewJob>,
) -> Result<impl IntoResponse, HandlerError> {
    db.add_job(employer_id, job).await?;

    Ok((StatusCode::CREATED, [("HX-Push-Url", "/dashboard/jobs/list")]))
}
