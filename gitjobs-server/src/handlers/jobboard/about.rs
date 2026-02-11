//! HTTP handlers for the about page.

use anyhow::{Result, anyhow};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use cached::proc_macro::cached;
use chrono::Duration;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    config::HttpServerConfig,
    handlers::{auth::AUTH_PROVIDER_KEY, error::HandlerError, prepare_headers},
    templates::{PageId, auth::User, jobboard::about::Page},
};

/// Handler that returns the about page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    session: Session,
    State(cfg): State<HttpServerConfig>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = Page {
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
        cfg: cfg.into(),
        content: prepare_content()?,
        page_id: PageId::About,
        user: User::default(),
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)))
}

/// Prepares and caches the about page content as HTML from Markdown source.
#[cached(
    key = "&str",
    convert = r#"{ "about_content" }"#,
    sync_writes = "by_key",
    result = true
)]
pub(crate) fn prepare_content() -> Result<String> {
    let md = include_str!("../../../../docs/about.md");
    let options = markdown::Options::gfm();
    let html = markdown::to_html_with_options(md, &options).map_err(|e| anyhow!(e))?;
    Ok(html)
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
        db::mock::MockDB, handlers::tests::TestRouterBuilder, notifications::MockNotificationsManager,
    };

    use super::*;

    #[tokio::test]
    async fn test_page_returns_html_with_cache_headers() {
        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/about")
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

    #[test]
    fn test_prepare_content_renders_html() {
        let html = prepare_content().unwrap();

        assert!(!html.is_empty());
        assert!(html.contains('<'));
    }
}
