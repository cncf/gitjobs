//! This module defines the templates for the dashboard home page.

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{error::HandlerError, extractors::SelectedEmployerIdOptional},
    templates::dashboard::{
        employers,
        home::{self, Content, Tab},
        jobs,
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
    let mut tab: Tab = query.get("tab").into();
    if employer_id.is_none() {
        tab = Tab::EmployerInitialSetup;
    }

    let content = match tab {
        Tab::EmployerInitialSetup => Content::EmployerInitialSetup(employers::InitialSetupPage {}),
        Tab::Jobs => {
            let jobs = db.list_employer_jobs(&employer_id.expect("to be some")).await?;
            Content::Jobs(jobs::ListPage { jobs })
        }
        Tab::Settings => {
            let employer_details = db.get_employer_details(&employer_id.expect("to be some")).await?;
            Content::Settings(employers::UpdatePage { employer_details })
        }
    };

    let user = auth_session.user.expect("user must be authenticated");
    let employers = db.get_user_employers(&user.user_id).await?;

    let template = home::Page {
        content,
        employers,
        selected_employer_id: employer_id,
    };

    Ok(Html(template.render()?))
}
