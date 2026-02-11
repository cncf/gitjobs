//! HTTP handlers for the jobs pages.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Json, Path, State},
    response::{Html, IntoResponse},
};
use chrono::Duration;
use reqwest::StatusCode;
use serde_qs::axum::QsQuery;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    db::{DynDB, jobboard::JobsSearchOutput},
    event_tracker::{DynEventTracker, Event},
    handlers::{auth::AUTH_PROVIDER_KEY, error::HandlerError, prepare_headers},
    templates::{
        PageId,
        auth::User,
        jobboard::jobs::{ExploreSection, Filters, JobSection, JobsPage, ResultsSection},
        pagination::{NavigationLinks, build_url},
    },
};

// Pages and sections handlers.

/// Returns the main jobs page with filters and results.
#[instrument(skip_all, err)]
pub(crate) async fn jobs_page(
    session: Session,
    State(db): State<DynDB>,
    State(cfg): State<HttpServerConfig>,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and jobs that match the query
    let (filters_options, JobsSearchOutput { jobs, total }) =
        tokio::try_join!(db.get_jobs_filters_options(), db.search_jobs(&filters))?;

    // Prepare template
    let template = JobsPage {
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        explore_section: ExploreSection {
            filters: filters.clone(),
            filters_options,
            results_section: ResultsSection {
                jobs,
                navigation_links: NavigationLinks::from_filters(&filters, total)?,
                total,
                offset: filters.offset,
            },
        },
        page_id: PageId::JobBoard,
        user: User::default(),
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::minutes(10), &[])?;

    Ok((headers, Html(template.render()?)))
}

/// Returns the results section for filtered jobs.
#[instrument(skip_all, err)]
pub(crate) async fn results_section(
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get jobs that match the query
    let JobsSearchOutput { jobs, total } = db.search_jobs(&filters).await?;

    // Prepare template
    let template = ResultsSection {
        navigation_links: NavigationLinks::from_filters(&filters, total)?,
        jobs,
        total,
        offset: filters.offset,
    };

    // Prepare response headers
    let url = build_url("/", &filters)?;
    let extra_headers = [("HX-Replace-Url", url.as_str())];
    let headers = prepare_headers(Duration::minutes(10), &extra_headers)?;

    Ok((headers, Html(template.render()?)))
}

/// Returns the job details section for a specific job.
#[instrument(skip_all, err)]
pub(crate) async fn job_section(
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get job information
    let Some(job) = db.get_job_jobboard(&job_id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // Prepare template
    let template = JobSection {
        base_url: cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url).to_string(),
        job,
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)).into_response())
}

// Actions handlers.

/// Allows an authenticated user to apply to a job.
#[instrument(skip_all, err)]
pub(crate) async fn apply(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN);
    };

    // Create job application entry in the database
    let applied = db.apply_to_job(&job_id, &user.user_id).await?;
    if !applied {
        return Ok(StatusCode::CONFLICT);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Tracks a view for a specific job in the job board.
#[instrument(skip_all, err)]
pub(crate) async fn track_view(
    State(event_tracker): State<DynEventTracker>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    event_tracker.track(Event::JobView { job_id }).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Tracks search appearances for multiple jobs.
#[instrument(skip_all, err)]
pub(crate) async fn track_search_appearances(
    State(event_tracker): State<DynEventTracker>,
    Json(job_ids): Json<Vec<Uuid>>,
) -> Result<impl IntoResponse, HandlerError> {
    event_tracker.track(Event::SearchAppearances { job_ids }).await?;

    Ok(StatusCode::NO_CONTENT)
}

// Tests.

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{
            Request, StatusCode,
            header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE},
        },
    };
    use axum_login::tower_sessions::session;
    use serde_json::json;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::mock::MockDB,
        event_tracker::MockEventTracker,
        handlers::tests::{
            TestRouterBuilder, expect_track_search_appearances, expect_track_view, sample_auth_user,
            sample_jobboard_filters_options, sample_jobboard_job, sample_jobboard_jobs_output,
            sample_session_record,
        },
        notifications::MockNotificationsManager,
    };

    #[tokio::test]
    async fn test_jobs_page_returns_html_with_results() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_jobs_filters_options()
            .times(1)
            .returning(|| Ok(sample_jobboard_filters_options()));
        db.expect_search_jobs()
            .times(1)
            .returning(move |_| Ok(sample_jobboard_jobs_output(job_id, employer_id)));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder().method("GET").uri("/").body(Body::empty()).unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CACHE_CONTROL], "max-age=0");
        assert_eq!(parts.headers[CONTENT_TYPE], "text/html; charset=utf-8");
        assert!(!bytes.is_empty());
    }

    #[tokio::test]
    async fn test_results_section_returns_html() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_search_jobs()
            .times(1)
            .returning(move |_| Ok(sample_jobboard_jobs_output(job_id, employer_id)));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/section/jobs/results")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CACHE_CONTROL], "max-age=0");
        assert_eq!(parts.headers[CONTENT_TYPE], "text/html; charset=utf-8");
        assert!(!bytes.is_empty());
    }

    #[tokio::test]
    async fn test_job_section_returns_not_found_when_job_is_missing() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_job_jobboard()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(|_| Ok(None));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/section/jobs/{job_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_job_section_returns_html_when_job_exists() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_job_jobboard()
            .times(1)
            .withf(move |id| *id == job_id)
            .returning(move |_| Ok(Some(sample_jobboard_job(job_id, employer_id))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/section/jobs/{job_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CACHE_CONTROL], "max-age=0");
        assert_eq!(parts.headers[CONTENT_TYPE], "text/html; charset=utf-8");
        assert!(!bytes.is_empty());
    }

    #[tokio::test]
    async fn test_apply_returns_no_content_when_application_is_created() {
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
        db.expect_apply_to_job()
            .times(1)
            .withf(move |id, user| *id == job_id && *user == user_id)
            .returning(|_, _| Ok(true));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri(format!("/jobs/{job_id}/apply"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_apply_returns_conflict_when_already_applied() {
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
        db.expect_apply_to_job()
            .times(1)
            .withf(move |id, user| *id == job_id && *user == user_id)
            .returning(|_, _| Ok(false));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri(format!("/jobs/{job_id}/apply"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_track_view_returns_no_content() {
        // Setup identifiers and data structures
        let job_id = Uuid::new_v4();

        // Setup event tracker mock
        let mut event_tracker = MockEventTracker::new();
        expect_track_view(&mut event_tracker, job_id);

        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_event_tracker(event_tracker)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri(format!("/jobs/{job_id}/views"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_track_search_appearances_returns_no_content() {
        // Setup identifiers and data structures
        let job_ids = vec![Uuid::new_v4(), Uuid::new_v4()];

        // Setup event tracker mock
        let mut event_tracker = MockEventTracker::new();
        expect_track_search_appearances(&mut event_tracker, job_ids.clone());

        // Setup router and send request
        let body = json!(job_ids).to_string();
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_event_tracker(event_tracker)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/jobs/search-appearances")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
