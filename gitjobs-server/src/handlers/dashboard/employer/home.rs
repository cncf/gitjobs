//! This module defines the HTTP handlers for the employer dashboard home page.

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    extract::{Query, RawQuery, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_messages::Messages;
use rinja::Template;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::{DynDB, dashboard::employer::ApplicationsSearchOutput},
    handlers::{error::HandlerError, extractors::SelectedEmployerIdOptional},
    templates::{
        PageId, auth,
        dashboard::employer::{
            applications, employers,
            home::{self, Content, Tab},
            jobs,
        },
    },
};

/// Handler that returns the employer dashboard home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    auth_session: AuthSession,
    messages: Messages,
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    Query(query): Query<HashMap<String, String>>,
    SelectedEmployerIdOptional(employer_id): SelectedEmployerIdOptional,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Get selected tab from query
    let mut tab: Tab = query.get("tab").into();
    if tab != Tab::Account && employer_id.is_none() {
        tab = Tab::EmployerInitialSetup;
    }

    // Prepare content for the selected tab
    let content = match tab {
        Tab::Account => {
            let user_summary = user.clone().into();
            Content::Account(auth::UpdateUserPage { user_summary })
        }
        Tab::Applications => {
            let employer_id = employer_id.expect("to be some");
            let filters = applications::Filters::new(&serde_qs_de, &raw_query.unwrap_or_default())?;
            let (
                filters_options,
                ApplicationsSearchOutput {
                    applications,
                    total: _,
                },
            ) = tokio::try_join!(
                db.get_applications_filters_options(&employer_id),
                db.search_applications(&employer_id, &filters)
            )?;
            Content::Applications(applications::ApplicationsPage {
                applications,
                filters,
                filters_options,
            })
        }
        Tab::EmployerInitialSetup => Content::EmployerInitialSetup(employers::InitialSetupPage {}),
        Tab::Jobs => {
            let jobs = db.list_employer_jobs(&employer_id.expect("to be some")).await?;
            Content::Jobs(jobs::ListPage { jobs })
        }
        Tab::Profile => {
            let employer = db.get_employer(&employer_id.expect("to be some")).await?;
            Content::Profile(employers::UpdatePage { employer })
        }
    };

    // Prepare template
    let employers = db.list_employers(&user.user_id).await?;
    let template = home::Page {
        content,
        employers,
        logged_in: true,
        messages: messages.into_iter().collect(),
        page_id: PageId::EmployerDashboard,
        name: Some(user.name),
        selected_employer_id: employer_id,
        username: Some(user.username),
    };

    Ok(Html(template.render()?).into_response())
}
