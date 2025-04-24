//! This module defines the HTTP handlers for the moderator dashboard home
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
    config::HttpServerConfig,
    db::DynDB,
    handlers::error::HandlerError,
    templates::{
        PageId,
        dashboard::{
            employer::jobs::JobStatus,
            moderator::{
                home::{self, Content, Tab},
                jobs,
            },
        },
    },
};

// Pages handlers.

/// Handler that returns the moderator dashboard home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    messages: Messages,
    State(db): State<DynDB>,
    State(cfg): State<HttpServerConfig>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(_user) = auth_session.user.clone() else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare content for the selected tab
    let tab: Tab = query.get("tab").unwrap_or(&String::new()).parse().unwrap_or_default();
    let content = match tab {
        Tab::LiveJobs => {
            let jobs = db.list_jobs_for_moderation(JobStatus::Published).await?;
            Content::LiveJobs(jobs::LivePage { jobs })
        }
        Tab::PendingJobs => {
            let jobs = db.list_jobs_for_moderation(JobStatus::PendingApproval).await?;
            Content::PendingJobs(jobs::PendingPage { jobs })
        }
    };

    // Prepare template
    let template = home::Page {
        cfg: cfg.into(),
        content,
        messages: messages.into_iter().collect(),
        page_id: PageId::ModeratorDashboard,
        user: auth_session.into(),
    };

    Ok(Html(template.render()?).into_response())
}
