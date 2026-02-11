//! HTTP handlers for job board embed endpoints, including jobs and job card embeds.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use chrono::Duration;
use serde_qs::axum::QsQuery;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::HttpServerConfig,
    db::{DynDB, jobboard::JobsSearchOutput},
    handlers::{error::HandlerError, prepare_headers},
    templates::jobboard::{
        embed::{JobCard, JobsPage},
        jobs::Filters,
    },
};

/// Returns the jobs embed page for external integration.
#[instrument(skip_all, err)]
pub(crate) async fn jobs_page(
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get jobs that match the query
    let JobsSearchOutput { jobs, total: _ } = db.search_jobs(&filters).await?;

    // Prepare template
    let template = JobsPage {
        base_url: cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url).to_string(),
        jobs,
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::minutes(10), &[])?;

    Ok((headers, Html(template.render()?)))
}

/// Returns the job card embed as an SVG image for sharing or embedding.
#[instrument(skip_all, err)]
pub(crate) async fn job_card(
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = JobCard {
        base_url: cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url).to_string(),
        job: db.get_job_jobboard(&job_id).await?,
    };

    // Prepare response headers
    let extra_headers = [("content-type", "image/svg+xml")];
    let headers = prepare_headers(Duration::minutes(10), &extra_headers)?;

    Ok((headers, template.render()?))
}

// Tests.

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{
            Request, StatusCode,
            header::{CACHE_CONTROL, CONTENT_TYPE},
        },
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::mock::MockDB,
        handlers::tests::{TestRouterBuilder, sample_jobboard_job, sample_jobboard_jobs_output},
        notifications::MockNotificationsManager,
    };

    #[tokio::test]
    async fn test_jobs_page_returns_html_embed() {
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
            .uri("/embed")
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
    async fn test_job_card_returns_svg() {
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
            .uri(format!("/embed/job/{job_id}/card.svg"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CACHE_CONTROL], "max-age=0");
        assert_eq!(parts.headers[CONTENT_TYPE], "image/svg+xml");
        assert!(!bytes.is_empty());
    }
}
