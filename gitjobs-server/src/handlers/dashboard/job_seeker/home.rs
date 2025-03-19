//! This module defines the HTTP handlers for the job seeker dashboard home
//! page.

use std::collections::HashMap;

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_messages::Messages;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    templates::{
        PageId, auth,
        dashboard::job_seeker::{
            home::{self, Content, Tab},
            profile,
        },
    },
};

// Pages handlers.

/// Handler that returns the job seeker dashboard home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    messages: Messages,
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare content for the selected tab
    let tab: Tab = query.get("tab").unwrap_or(&String::new()).parse().unwrap_or_default();
    let content = match tab {
        Tab::Account => {
            let user_summary = user.clone().into();
            Content::Account(auth::UpdateUserPage { user_summary })
        }
        Tab::Profile => {
            let profile = db.get_job_seeker_profile(&user.user_id).await?;
            Content::Profile(profile::UpdatePage { profile })
        }
    };

    // Prepare template
    let template = home::Page {
        content,
        logged_in: true,
        messages: messages.into_iter().collect(),
        name: Some(user.name),
        page_id: PageId::JobSeekerDashboard,
        username: Some(user.username),
    };

    Ok(Html(template.render()?).into_response())
}
