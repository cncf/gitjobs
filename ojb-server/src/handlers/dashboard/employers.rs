//! This module defines the HTTP handlers to manage employers.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use rinja::Template;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{JobBoardId, SelectedEmployerIdRequired},
    },
    templates::dashboard::employers,
};

// Pages handlers.

/// Handler that returns the page to add a new employer.
#[instrument(skip_all, err)]
pub(crate) async fn add_page(State(_db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let template = employers::AddPage {};

    Ok(Html(template.render()?))
}

/// Handler that returns the page to update an employer.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let employer_details = db.get_employer_details(&employer_id).await?;
    let template = employers::UpdatePage { employer_details };

    Ok(Html(template.render()?))
}

// Actions handlers.

/// Handler that adds an employer.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    auth_session: AuthSession,
    session: Session,
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
    Form(employer_details): Form<employers::EmployerDetails>,
) -> Result<impl IntoResponse, HandlerError> {
    // Check if the user is logged in
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN);
    };

    // Add employer to database
    let employer_id = db
        .add_employer(&job_board_id, &user.user_id, &employer_details)
        .await?;

    // Use new employer as the selected employer for the session
    session.insert("selected_employer_id", employer_id).await?;

    Ok(StatusCode::CREATED)
}

/// Handler that selects an employer.
#[instrument(skip_all, err)]
pub(crate) async fn select(
    session: Session,
    Path(employer_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // TODO: check if the user is part of the employer's team

    session.insert("selected_employer_id", employer_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that updates an employer.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    Form(employer_details): Form<employers::EmployerDetails>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_employer(&employer_id, &employer_details).await?;

    Ok(StatusCode::NO_CONTENT)
}
