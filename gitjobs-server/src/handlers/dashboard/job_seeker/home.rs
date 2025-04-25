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
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    db::DynDB,
    handlers::{auth::AUTH_PROVIDER_KEY, error::HandlerError},
    templates::{
        PageId, auth,
        dashboard::job_seeker::{
            applications,
            home::{self, Content, Tab},
            profile,
        },
    },
};

// Pages handlers.

/// Handler that returns the moderator dashboard home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    messages: Messages,
    session: Session,
    State(db): State<DynDB>,
    State(cfg): State<HttpServerConfig>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user.clone() else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare content for the selected tab
    let tab: Tab = query.get("tab").unwrap_or(&String::new()).parse().unwrap_or_default();
    let content = match tab {
        Tab::Account => {
            let user_summary = user.clone().into();
            Content::Account(auth::UpdateUserPage { user_summary })
        }
        Tab::Applications => {
            let applications = db.list_job_seeker_applications(&user.user_id).await?;
            Content::Applications(applications::ApplicationsPage { applications })
        }
        Tab::Profile => {
            let profile = db.get_job_seeker_profile(&user.user_id).await?;
            Content::Profile(profile::UpdatePage { profile })
        }
    };

    // Prepare template
    let template = home::Page {
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        content,
        messages: messages.into_iter().collect(),
        page_id: PageId::JobSeekerDashboard,
        user: auth_session.into(),
    };

    Ok(Html(template.render()?).into_response())
}
