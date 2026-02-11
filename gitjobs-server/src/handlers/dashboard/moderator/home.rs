//! This module defines the HTTP handlers for the moderator dashboard home page.

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
///
/// This function handles the HTTP request for the moderator dashboard home page.
/// It retrieves the user from the session, determines the selected tab, fetches
/// the relevant data from the database, and renders the appropriate template.
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
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        content,
        messages: messages.into_iter().collect(),
        page_id: PageId::ModeratorDashboard,
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
            TestRouterBuilder, sample_auth_user, sample_moderator_job_summary, sample_session_record,
        },
        notifications::MockNotificationsManager,
        templates::dashboard::employer::jobs::JobStatus,
    };

    #[tokio::test]
    async fn test_page_renders_live_jobs_tab_for_moderator() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let mut moderator = sample_auth_user(user_id, auth_hash);
        moderator.moderator = true;

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(moderator.clone())));
        db.expect_list_jobs_for_moderation()
            .times(1)
            .withf(|status| *status == JobStatus::Published)
            .returning(move |_| Ok(vec![sample_moderator_job_summary(job_id, employer_id)]));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/dashboard/moderator?tab=live-jobs")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }
}
