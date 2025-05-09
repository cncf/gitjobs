//! This module defines the HTTP handlers for the employer dashboard team page.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use axum_extra::extract::Form;
use reqwest::StatusCode;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{error::HandlerError, extractors::SelectedEmployerIdRequired},
    templates::dashboard::employer::team::{self, NewTeamMember},
};

// Pages handlers.

/// Handler that returns the page to add a new team member.
#[instrument(skip_all, err)]
pub(crate) async fn add_member_page(State(_db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let template = team::AddMemberPage {};

    Ok(Html(template.render()?))
}

/// Handler that returns the team members list page.
#[instrument(skip_all, err)]
pub(crate) async fn members_list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let team_members = db.list_team_members(&employer_id).await?;
    let template = team::MembersListPage { team_members };

    Ok(Html(template.render()?))
}

// Actions handlers.

/// Handler that adds a new team member.
#[instrument(skip_all, err)]
pub(crate) async fn add_member(
    State(db): State<DynDB>,
    Form(member): Form<NewTeamMember>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    // Add the new team member to the database
    db.add_team_member(&employer_id, &member.email).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that deletes a team member.
#[instrument(skip_all, err)]
pub(crate) async fn delete_member(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Delete the team member from the database
    db.delete_team_member(&employer_id, &user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that accepts a team member invitation.
#[instrument(skip_all, err)]
pub(crate) async fn accept_invitation(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Path(employer_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN);
    };

    // Update the team member status in the database (invitation accepted)
    db.accept_team_member_invitation(&employer_id, &user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handler that rejects a team member invitation.
#[instrument(skip_all, err)]
pub(crate) async fn reject_invitation(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Path(employer_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN);
    };

    // Delete the team member from the database
    db.delete_team_member(&employer_id, &user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
