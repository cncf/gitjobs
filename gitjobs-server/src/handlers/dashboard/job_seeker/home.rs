//! This module defines the HTTP handlers for the job seeker dashboard home page.

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

/// Handler that returns the job seeker dashboard home page.
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

// Tests.

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::COOKIE},
    };
    use axum_login::tower_sessions::session;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::mock::MockDB,
        handlers::tests::{
            TestRouterBuilder, sample_auth_user, sample_job_seeker_application, sample_session_record,
        },
        notifications::MockNotificationsManager,
    };

    #[tokio::test]
    async fn test_page_renders_applications_tab() {
        // Setup identifiers and data structures
        let application_id = Uuid::new_v4();
        let auth_hash = "hash";
        let job_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_list_job_seeker_applications()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(vec![sample_job_seeker_application(application_id, job_id)]));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/dashboard/job-seeker?tab=applications")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }
}
