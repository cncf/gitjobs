//! This module defines the HTTP handlers for the applications page.

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession, db::DynDB, handlers::error::HandlerError,
    templates::dashboard::job_seeker::applications::ApplicationsPage,
};

// Pages handlers.

/// Handler that returns the applications list page.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare template
    let applications = db.list_job_seeker_applications(&user.user_id).await?;
    let template = ApplicationsPage { applications };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Handler that cancels an application.
#[instrument(skip_all, err)]
pub(crate) async fn cancel(
    auth_session: AuthSession,
    Path(application_id): Path<Uuid>,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok((StatusCode::FORBIDDEN).into_response());
    };

    // Cancel application
    db.cancel_application(&application_id, &user.user_id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [(
            "HX-Location",
            r#"{"path":"/dashboard/job-seeker?tab=applications", "target":"body"}"#,
        )],
    )
        .into_response())
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
    async fn test_list_page_renders_successfully() {
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
            .uri("/dashboard/job-seeker/applications/list")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cancel_returns_no_content() {
        // Setup identifiers and data structures
        let application_id = Uuid::new_v4();
        let auth_hash = "hash";
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
        db.expect_cancel_application()
            .times(1)
            .withf(move |id, user| *id == application_id && *user == user_id)
            .returning(|_, _| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!(
                "/dashboard/job-seeker/applications/{application_id}/cancel"
            ))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
