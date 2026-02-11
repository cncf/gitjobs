//! This module defines the HTTP handlers for previewing, updating, and saving job
//! seeker profiles.

use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::error::HandlerError,
    templates::dashboard::job_seeker::profile::{self, JobSeekerProfile},
};

// Pages handlers.

/// Handler that returns the page to preview a profile.
#[instrument(skip_all, err)]
pub(crate) async fn preview_page(
    State(serde_qs_de): State<serde_qs::Config>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get profile information from body
    let mut profile: JobSeekerProfile = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    profile.normalize();

    // Prepare template
    let template = profile::PreviewPage { profile };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the page to update a profile.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    auth_session: AuthSession,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Prepare template
    let profile = db.get_job_seeker_profile(&user.user_id).await?;
    let template = profile::UpdatePage { profile };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Handler that updates a job seeker's profile in the database.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    auth_session: AuthSession,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Get profile information from body
    let mut profile: JobSeekerProfile = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    profile.normalize();

    // Update profile in database
    db.update_job_seeker_profile(&user.user_id, &profile).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

// Tests.

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode, header::COOKIE},
        response::IntoResponse,
    };
    use axum_login::tower_sessions::session;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::mock::MockDB,
        handlers::tests::{
            TestRouterBuilder, qs_config, sample_auth_user, sample_job_seeker_profile, sample_session_record,
        },
        notifications::MockNotificationsManager,
    };

    use super::*;

    #[tokio::test]
    async fn test_preview_page_returns_unprocessable_entity_for_invalid_body() {
        // Execute handler
        let response = preview_page(State(qs_config()), "invalid-body".to_string())
            .await
            .unwrap()
            .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_preview_page_renders_successfully() {
        // Setup identifiers and data structures
        let profile = sample_job_seeker_profile();
        let body = serde_qs::to_string(&profile).unwrap();

        // Execute handler
        let response = preview_page(State(qs_config()), body).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_page_renders_successfully() {
        // Setup identifiers and data structures
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
        db.expect_get_job_seeker_profile()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(|_| Ok(Some(sample_job_seeker_profile())));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/dashboard/job-seeker/profile/update")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_returns_no_content() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let profile = sample_job_seeker_profile();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let body = serde_qs::to_string(&profile).unwrap();

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
        db.expect_update_job_seeker_profile()
            .times(1)
            .withf(move |id, _| *id == user_id)
            .returning(|_, _| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri("/dashboard/job-seeker/profile/update")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
