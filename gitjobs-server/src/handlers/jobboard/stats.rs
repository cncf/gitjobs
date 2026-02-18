//! HTTP handlers for the stats page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use chrono::Duration;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    config::HttpServerConfig,
    db::DynDB,
    handlers::{auth::AUTH_PROVIDER_KEY, error::HandlerError, prepare_headers},
    templates::{PageId, auth::User, jobboard::stats::Page},
};

/// Handler that returns the stats page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    session: Session,
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get stats information from the database
    let stats = db.get_stats().await?;

    // Prepare template
    let template = Page {
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        page_id: PageId::Stats,
        stats,
        user: User::default(),
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)))
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

    use crate::{
        db::mock::MockDB,
        handlers::tests::{TestRouterBuilder, sample_jobboard_stats},
        notifications::MockNotificationsManager,
    };

    #[tokio::test]
    async fn test_page_returns_html_with_stats() {
        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_stats()
            .times(1)
            .returning(|| Ok(sample_jobboard_stats()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/stats")
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
}
