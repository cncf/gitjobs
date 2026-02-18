//! This module defines the HTTP handlers for the jobs moderation dashboard pages.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use garde::Validate;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    db::DynDB,
    handlers::{error::HandlerError, extractors::ValidatedForm},
    templates::{
        dashboard::{
            employer::{self, jobs::JobStatus},
            moderator::jobs,
        },
        helpers::{empty_string_as_none, option_is_none_or_default},
        notifications::JobPublished,
    },
    validation::{MAX_LEN_DESCRIPTION_SHORT, trimmed_non_empty_opt},
};

// Pages handlers.

/// Returns the page listing all live (published) jobs for moderation.
#[instrument(skip_all, err)]
pub(crate) async fn live_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_jobs_for_moderation(JobStatus::Published).await?;
    let template = jobs::LivePage { jobs };

    Ok(Html(template.render()?))
}

/// Returns the page listing all jobs pending approval for moderation.
#[instrument(skip_all, err)]
pub(crate) async fn pending_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_jobs_for_moderation(JobStatus::PendingApproval).await?;
    let template = jobs::PendingPage { jobs };

    Ok(Html(template.render()?))
}

/// Returns the preview page for a specific job and its employer.
#[instrument(skip_all, err)]
pub(crate) async fn preview_page(
    State(db): State<DynDB>,
    Path((employer_id, job_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HandlerError> {
    let (employer, job) = tokio::try_join!(db.get_employer(&employer_id), db.get_job_dashboard(&job_id))?;
    let template = employer::jobs::PreviewPage { employer, job };

    Ok(Html(template.render()?).into_response())
}

// Actions.

/// Approves a job as a moderator and triggers a table refresh in the UI.
#[instrument(skip_all, err)]
pub(crate) async fn approve(
    auth_session: AuthSession,
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    State(http_client): State<Client>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Approve job
    let previous_first_published_at = db.approve_job(&job_id, &user.user_id).await?;

    // Post a Slack notification the first time a job is published
    if previous_first_published_at.is_none()
        && let Some(webhook_url) = &cfg.slack_webhook_url
        && let Some(job) = db.get_job_jobboard(&job_id).await?
    {
        let template = JobPublished {
            base_url: cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url).to_string(),
            job,
        };
        let payload = json!({
            "text": template.render()?,
        });
        if let Err(err) = http_client.post(webhook_url).json(&payload).send().await {
            warn!("error posting slack notification: {}", err);
        }
    }

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-moderator-table")],
    )
        .into_response())
}

/// Rejects a job as a moderator, optionally including review notes, and triggers a table
/// refresh.
#[instrument(skip_all, err)]
pub(crate) async fn reject(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    ValidatedForm(input): ValidatedForm<RejectInput>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Reject job
    db.reject_job(&job_id, &user.user_id, input.review_notes).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("HX-Trigger", "refresh-moderator-table")],
    )
        .into_response())
}

// Types.

/// Input data for rejecting a job, including optional review notes.
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub(crate) struct RejectInput {
    /// Optional review notes provided by the moderator when rejecting a job.
    #[garde(custom(trimmed_non_empty_opt), length(max = MAX_LEN_DESCRIPTION_SHORT))]
    #[serde(
        default,
        deserialize_with = "empty_string_as_none",
        skip_serializing_if = "option_is_none_or_default"
    )]
    pub review_notes: Option<String>,
}

// Tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::{Path, State},
        http::{Request, StatusCode, header::COOKIE},
        response::IntoResponse,
    };
    use axum_login::tower_sessions::session;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::{DynDB, mock::MockDB},
        handlers::tests::{
            TestRouterBuilder, sample_auth_user, sample_employer, sample_employer_job, sample_jobboard_job,
            sample_moderator_job_summary, sample_session_record, test_http_server_cfg,
        },
        notifications::MockNotificationsManager,
        templates::dashboard::employer::jobs::JobStatus,
    };

    use super::*;

    #[tokio::test]
    async fn test_live_page_renders_successfully() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_list_jobs_for_moderation()
            .times(1)
            .withf(|status| *status == JobStatus::Published)
            .returning(move |_| Ok(vec![sample_moderator_job_summary(job_id, employer_id)]));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = live_page(State(db)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_pending_page_renders_successfully() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_list_jobs_for_moderation()
            .times(1)
            .withf(|status| *status == JobStatus::PendingApproval)
            .returning(move |_| Ok(vec![sample_moderator_job_summary(job_id, employer_id)]));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = pending_page(State(db)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_preview_page_renders_successfully() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_employer()
            .times(1)
            .withf(move |id| *id == employer_id)
            .returning(move |_| Ok(sample_employer(employer_id)));
        db.expect_get_job_dashboard()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(sample_employer_job(job_id)));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = preview_page(State(db), Path((employer_id, job_id)))
            .await
            .unwrap()
            .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_approve_returns_no_content_for_moderator() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
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
        db.expect_approve_job()
            .times(1)
            .withf(move |id, reviewer| *id == job_id && *reviewer == user_id)
            .returning(|_, _| Ok(Some(chrono::Utc::now())));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/moderator/jobs/{job_id}/approve"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_approve_checks_job_data_for_first_publication_when_webhook_is_enabled() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let mut moderator = sample_auth_user(user_id, auth_hash);
        moderator.moderator = true;
        let mut cfg = test_http_server_cfg();
        cfg.slack_webhook_url = Some("http://127.0.0.1:9/webhook".to_string());

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
        db.expect_approve_job()
            .times(1)
            .withf(move |id, reviewer| *id == job_id && *reviewer == user_id)
            .returning(|_, _| Ok(None));
        db.expect_get_job_jobboard()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(sample_jobboard_job(job_id, employer_id))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/moderator/jobs/{job_id}/approve"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_reject_returns_no_content_for_moderator() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
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
        db.expect_reject_job()
            .times(1)
            .withf(move |id, reviewer, notes| {
                *id == job_id && *reviewer == user_id && notes.as_deref() == Some("missing details")
            })
            .returning(|_, _, _| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/moderator/jobs/{job_id}/reject"))
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("review_notes=missing+details"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_reject_passes_none_review_notes_when_not_provided() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
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
        db.expect_reject_job()
            .times(1)
            .withf(move |id, reviewer, notes| *id == job_id && *reviewer == user_id && notes.is_none())
            .returning(|_, _, _| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/moderator/jobs/{job_id}/reject"))
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_reject_passes_none_review_notes_when_provided_as_blank_string() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
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
        db.expect_reject_job()
            .times(1)
            .withf(move |id, reviewer, notes| *id == job_id && *reviewer == user_id && notes.is_none())
            .returning(|_, _, _| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/moderator/jobs/{job_id}/reject"))
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("review_notes="))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_reject_returns_unprocessable_entity_for_invalid_review_notes() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
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
        db.expect_reject_job().times(0);
        db.expect_update_session().times(0..).returning(|_| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/moderator/jobs/{job_id}/reject"))
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("review_notes=+"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
