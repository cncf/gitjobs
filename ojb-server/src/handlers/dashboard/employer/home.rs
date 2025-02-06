//! This module defines the HTTP handlers for the employer dashboard home page.

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
    handlers::{error::HandlerError, extractors::SelectedEmployerIdOptional},
    templates::dashboard::employer::{
        self,
        home::{self, Content, Tab},
    },
};

/// Handler that returns the dashboard home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
    SelectedEmployerIdOptional(employer_id): SelectedEmployerIdOptional,
) -> Result<impl IntoResponse, HandlerError> {
    // Check if the user is logged in
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Get selected tab from query
    let mut tab: Tab = query.get("tab").into();
    if employer_id.is_none() {
        tab = Tab::EmployerInitialSetup;
    }

    // Prepare content for the selected tab
    let content = match tab {
        Tab::EmployerInitialSetup => Content::EmployerInitialSetup(employer::employers::InitialSetupPage {}),
        Tab::Jobs => {
            let jobs = db.list_employer_jobs(&employer_id.expect("to be some")).await?;
            Content::Jobs(employer::jobs::ListPage { jobs })
        }
        Tab::Settings => {
            let employer_details = db.get_employer_details(&employer_id.expect("to be some")).await?;
            Content::Settings(employer::employers::UpdatePage { employer_details })
        }
    };

    // Prepare template
    let employers = db.list_employers(&user.user_id).await?;
    let template = home::Page {
        content,
        employers,
        selected_employer_id: employer_id,
    };

    Ok(Html(template.render()?).into_response())
}
