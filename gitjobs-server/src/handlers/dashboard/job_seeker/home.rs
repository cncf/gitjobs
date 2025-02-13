//! This module defines the HTTP handlers for the job seeker dashboard home
//! page.

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    templates::{
        auth,
        dashboard::job_seeker::{
            home::{self, Content, Tab},
            profile,
        },
    },
};

/// Handler that returns the job seeker dashboard home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare content for the selected tab
    let tab: Tab = query.get("tab").into();
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
        name: user.name,
        username: user.username,
    };

    Ok(Html(template.render()?).into_response())
}
