//! This module defines the HTTP handlers to manage employers.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_extra::extract::Form;
use rinja::Template;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{
        auth::SELECTED_EMPLOYER_ID_KEY,
        error::HandlerError,
        extractors::{JobBoardId, SelectedEmployerIdRequired},
    },
    templates::dashboard::employer::employers,
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
    let employer = db.get_employer(&employer_id).await?;
    let template = employers::UpdatePage { employer };

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
    Form(employer): Form<employers::Employer>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok((StatusCode::FORBIDDEN).into_response());
    };

    // Add employer to database
    let employer_id = db.add_employer(&job_board_id, &user.user_id, &employer).await?;

    // Use new employer as the selected employer for the session
    session.insert(SELECTED_EMPLOYER_ID_KEY, employer_id).await?;

    Ok((StatusCode::CREATED, [("HX-Refresh", "true")]).into_response())
}

/// Handler that selects an employer.
#[instrument(skip_all, err)]
pub(crate) async fn select(
    session: Session,
    Path(employer_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Update the selected employer in the session
    session.insert(SELECTED_EMPLOYER_ID_KEY, employer_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that updates an employer.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    Form(employer): Form<employers::Employer>,
) -> Result<impl IntoResponse, HandlerError> {
    db.update_employer(&employer_id, &employer).await?;

    Ok(StatusCode::NO_CONTENT)
}
