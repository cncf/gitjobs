//! HTTP handlers for the employer dashboard home page, including tab content logic.

use std::collections::HashMap;

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_messages::Messages;
use serde_qs::axum::QsQuery;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    db::{DynDB, dashboard::employer::ApplicationsSearchOutput},
    handlers::{auth::AUTH_PROVIDER_KEY, error::HandlerError, extractors::SelectedEmployerIdOptional},
    templates::{
        PageId, auth,
        dashboard::employer::{
            applications, employers,
            home::{self, Content, Tab},
            jobs, team,
        },
        pagination::NavigationLinks,
    },
};

// Pages handlers.

/// Handler that returns the employer dashboard home page.
///
/// This handler manages the main employer dashboard page, selecting the appropriate tab
/// and preparing the content for each dashboard section, such as account, applications,
/// invitations, jobs, profile, and team.
#[instrument(skip_all, err)]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn page(
    auth_session: AuthSession,
    messages: Messages,
    session: Session,
    State(db): State<DynDB>,
    State(cfg): State<HttpServerConfig>,
    Query(query): Query<HashMap<String, String>>,
    QsQuery(filters): QsQuery<applications::Filters>,
    SelectedEmployerIdOptional(employer_id): SelectedEmployerIdOptional,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user.clone() else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Get employers and pending invitations
    let (employers, pending_invitations) = tokio::try_join!(
        db.list_employers(&user.user_id),
        db.get_user_invitations_count(&user.user_id)
    )?;

    // Get selected tab from query
    let mut tab: Tab = query.get("tab").unwrap_or(&String::new()).parse().unwrap_or_default();
    if (tab != Tab::Account && tab != Tab::Invitations) && employer_id.is_none() {
        if pending_invitations > 0 {
            tab = Tab::Invitations;
        } else {
            tab = Tab::EmployerInitialSetup;
        }
    }

    // Prepare content for the selected tab
    let content = match tab {
        Tab::Account => {
            let user_summary = user.clone().into();
            Content::Account(auth::UpdateUserPage { user_summary })
        }
        Tab::Applications => {
            let employer_id = employer_id.expect("to be some");
            let (filters_options, ApplicationsSearchOutput { applications, total }) = tokio::try_join!(
                db.get_applications_filters_options(&employer_id),
                db.search_applications(&employer_id, &filters)
            )?;
            let navigation_links = NavigationLinks::from_filters(&filters, total)?;
            Content::Applications(applications::ApplicationsPage {
                applications,
                filters,
                filters_options,
                navigation_links,
            })
        }
        Tab::EmployerInitialSetup => Content::EmployerInitialSetup(employers::InitialSetupPage {}),
        Tab::Invitations => {
            let invitations = db.list_user_invitations(&user.user_id).await?;
            Content::Invitations(team::UserInvitationsListPage { invitations })
        }
        Tab::Jobs => {
            let jobs = db.list_employer_jobs(&employer_id.expect("to be some")).await?;
            Content::Jobs(jobs::ListPage { jobs })
        }
        Tab::Profile => {
            let employer = db.get_employer(&employer_id.expect("to be some")).await?;
            let foundations = db.list_foundations().await?;
            Content::Profile(employers::UpdatePage {
                employer,
                foundations,
            })
        }
        Tab::Team => {
            let members = db.list_team_members(&employer_id.expect("to be some")).await?;
            Content::Team(team::MembersListPage {
                approved_members_count: members.iter().filter(|m| m.approved).count(),
                members,
            })
        }
    };

    // Prepare template
    let template = home::Page {
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        content,
        employers,
        messages: messages.into_iter().collect(),
        page_id: PageId::EmployerDashboard,
        pending_invitations,
        selected_employer_id: employer_id,
        user: auth_session.into(),
    };

    Ok(Html(template.render()?).into_response())
}
