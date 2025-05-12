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
    config::HttpServerConfig,
    db::DynDB,
    handlers::{error::HandlerError, extractors::SelectedEmployerIdRequired},
    notifications::{DynNotificationsManager, NewNotification, NotificationKind},
    templates::{
        dashboard::employer::team::{self, NewTeamMember},
        notifications::TeamInvitation,
    },
};

// Pages handlers.

/// Handler that returns the team members list page.
#[instrument(skip_all, err)]
pub(crate) async fn members_list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let members = db.list_team_members(&employer_id).await?;
    let template = team::MembersListPage { members };

    Ok(Html(template.render()?))
}

/// Handler that returns the user invitations list page.
#[instrument(skip_all, err)]
pub(crate) async fn user_invitations_list_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare template
    let invitations = db.list_user_invitations(&user.user_id).await?;
    let template = team::UserInvitationsListPage { invitations };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Handler that adds a new team member.
#[instrument(skip_all, err)]
pub(crate) async fn add_member(
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    State(notifications_manager): State<DynNotificationsManager>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    Form(member): Form<NewTeamMember>,
) -> Result<impl IntoResponse, HandlerError> {
    // Add the new team member to the database
    let user_id = db.add_team_member(&employer_id, &member.email).await?;

    // Enqueue team invitation notification (if member was added)
    if let Some(user_id) = user_id {
        let template_data = TeamInvitation {
            link: format!(
                "{}/dashboard/employer?tab=invitations",
                cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url)
            ),
        };
        let notification = NewNotification {
            kind: NotificationKind::TeamInvitation,
            user_id,
            template_data: Some(serde_json::to_value(&template_data)?),
        };
        notifications_manager.enqueue(&notification).await?;
    }

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

    // Mark team member as approved in the database
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
