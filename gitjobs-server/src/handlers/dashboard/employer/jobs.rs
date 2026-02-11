//! HTTP handlers for employer job management pages and actions.
//
// This module provides handlers for adding, listing, previewing, updating, archiving,
// deleting, and publishing jobs for employers in the dashboard. It also renders the
// corresponding Askama templates for each page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::{Duration, Utc};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::DynDB,
    handlers::{error::HandlerError, extractors::SelectedEmployerIdRequired, prepare_headers},
    templates::dashboard::employer::jobs::{self, Job, JobStatus},
};

// Pages handlers.

/// Renders the page to add a new job for an employer.
#[instrument(skip_all, err)]
pub(crate) async fn add_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let (certifications, foundations) = tokio::try_join!(db.list_certifications(), db.list_foundations())?;
    let template = jobs::AddPage {
        certifications,
        foundations,
    };

    Ok(Html(template.render()?))
}

/// Renders the jobs list page for the selected employer.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let jobs = db.list_employer_jobs(&employer_id).await?;
    let template = jobs::ListPage { jobs };

    Ok(Html(template.render()?))
}

/// Handler that returns the job preview page (job provided in body).
#[instrument(skip_all, err)]
pub(crate) async fn preview_page_w_job(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize().await;
    job.published_at = Some(Utc::now());
    job.updated_at = Some(Utc::now());

    // Prepare template
    let employer = db.get_employer(&employer_id).await?;
    let template = jobs::PreviewPage { employer, job };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the job preview page (job not provided in body).
#[instrument(skip_all, err)]
pub(crate) async fn preview_page_wo_job(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let (employer, job) = tokio::try_join!(db.get_employer(&employer_id), db.get_job_dashboard(&job_id))?;
    let template = jobs::PreviewPage { employer, job };

    Ok(Html(template.render()?).into_response())
}

/// Renders the page to update an existing job.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let (certifications, foundations, job) = tokio::try_join!(
        db.list_certifications(),
        db.list_foundations(),
        db.get_job_dashboard(&job_id)
    )?;
    let template = jobs::UpdatePage {
        certifications,
        foundations,
        job,
    };

    Ok(Html(template.render()?).into_response())
}

// Actions handlers.

/// Adds a new job for the selected employer.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize().await;

    // Make sure the status provided is valid
    if job.status != JobStatus::Draft && job.status != JobStatus::PendingApproval {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, "invalid status").into_response());
    }

    // Add job to database
    db.add_job(&employer_id, &job).await?;

    Ok((StatusCode::CREATED, [("HX-Trigger", "refresh-jobs-table")]).into_response())
}

/// Archives a job, making it inactive but not deleting it.
#[instrument(skip_all, err)]
pub(crate) async fn archive(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.archive_job(&job_id).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]))
}

/// Permanently deletes a job from the database.
#[instrument(skip_all, err)]
pub(crate) async fn delete(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.delete_job(&job_id).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]))
}

/// Publishes a job. It'll be visible to users once it's approved.
#[instrument(skip_all, err)]
pub(crate) async fn publish(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    db.publish_job(&job_id).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]))
}

/// Returns statistics for a specific job.
#[instrument(skip_all, err)]
pub(crate) async fn stats(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get stats information from the database
    let stats = db.get_job_stats(&job_id).await?;

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Json(stats)))
}

/// Updates an existing job with new data.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    Path(job_id): Path<Uuid>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information from body
    let mut job: Job = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };
    job.normalize().await;

    // Make sure the status provided is valid
    if job.status != JobStatus::Archived
        && job.status != JobStatus::Draft
        && job.status != JobStatus::PendingApproval
    {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, "invalid status").into_response());
    }

    // Update job in database
    db.update_job(&job_id, &job).await?;

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-jobs-table")]).into_response())
}

// Tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::{Path, State},
        http::{Request, header::COOKIE},
        response::IntoResponse,
    };
    use axum_login::tower_sessions::session;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::{DynDB, mock::MockDB},
        handlers::tests::{
            TestRouterBuilder, qs_config, sample_auth_user, sample_certifications, sample_employer,
            sample_employer_job, sample_employer_job_summary, sample_foundations, sample_job_stats,
            sample_session_record,
        },
        notifications::MockNotificationsManager,
    };

    use super::*;

    #[tokio::test]
    async fn test_add_page_renders_successfully() {
        // Setup database mock
        let mut db = MockDB::new();
        db.expect_list_certifications()
            .times(1)
            .returning(|| Ok(sample_certifications()));
        db.expect_list_foundations()
            .times(1)
            .returning(|| Ok(sample_foundations()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = add_page(State(db)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_add_rejects_invalid_status() {
        // Setup database mock
        let db: DynDB = Arc::new(MockDB::new());

        // Execute handler
        let response = add(
            State(db),
            State(qs_config()),
            crate::handlers::extractors::SelectedEmployerIdRequired(Uuid::new_v4()),
            "description=desc&kind=full-time&status=published&title=title&workplace=remote".to_string(),
        )
        .await
        .unwrap()
        .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_add_returns_created_for_valid_job() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job = sample_employer_job(Uuid::new_v4());

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_add_job()
            .times(1)
            .withf(move |id, _job| *id == employer_id)
            .returning(|_, _| Ok(()));
        let db: DynDB = Arc::new(db);
        let body = serde_qs::to_string(&job).unwrap();

        // Execute handler
        let response = add(
            State(db),
            State(qs_config()),
            crate::handlers::extractors::SelectedEmployerIdRequired(employer_id),
            body,
        )
        .await
        .unwrap()
        .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_archive_returns_no_content() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_archive_job()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(|_| Ok(()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = archive(State(db), Path(job_id)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_archive_route_checks_ownership_middleware() {
        // Setup identifiers and data structures
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
        db.expect_user_owns_job()
            .times(1)
            .withf(move |id, job| *id == user_id && *job == job_id)
            .returning(|_, _| Ok(true));
        db.expect_archive_job()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(|_| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/employer/jobs/{job_id}/archive"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_returns_no_content() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_delete_job()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(|_| Ok(()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = delete(State(db), Path(job_id)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_list_page_renders_successfully() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_list_employer_jobs()
            .times(1)
            .withf(move |id| *id == employer_id)
            .returning(move |_| Ok(vec![sample_employer_job_summary(job_id)]));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = list_page(
            State(db),
            crate::handlers::extractors::SelectedEmployerIdRequired(employer_id),
        )
        .await
        .unwrap()
        .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_preview_page_w_job_renders_successfully() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job = sample_employer_job(Uuid::new_v4());

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_employer()
            .times(1)
            .withf(move |id| *id == employer_id)
            .returning(move |_| Ok(sample_employer(employer_id)));
        let db: DynDB = Arc::new(db);
        let body = serde_qs::to_string(&job).unwrap();

        // Execute handler
        let response = preview_page_w_job(
            State(db),
            State(qs_config()),
            crate::handlers::extractors::SelectedEmployerIdRequired(employer_id),
            body,
        )
        .await
        .unwrap()
        .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_preview_page_wo_job_renders_successfully() {
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
        let response = preview_page_wo_job(
            State(db),
            Path(job_id),
            crate::handlers::extractors::SelectedEmployerIdRequired(employer_id),
        )
        .await
        .unwrap()
        .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_publish_returns_no_content() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_publish_job()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(|_| Ok(()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = publish(State(db), Path(job_id)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_stats_returns_json() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_job_stats()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(|_| Ok(sample_job_stats()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = stats(State(db), Path(job_id)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_page_renders_successfully() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_list_certifications()
            .times(1)
            .returning(|| Ok(sample_certifications()));
        db.expect_list_foundations()
            .times(1)
            .returning(|| Ok(sample_foundations()));
        db.expect_get_job_dashboard()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(sample_employer_job(job_id)));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = update_page(State(db), Path(job_id)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_returns_no_content_for_valid_job() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();
        let job = sample_employer_job(job_id);

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_update_job()
            .times(1)
            .withf(move |id, _job| *id == job_id)
            .returning(|_, _| Ok(()));
        let db: DynDB = Arc::new(db);
        let body = serde_qs::to_string(&job).unwrap();

        // Execute handler
        let response = update(State(db), State(qs_config()), Path(job_id), body)
            .await
            .unwrap()
            .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
