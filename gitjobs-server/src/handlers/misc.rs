//! HTTP handlers for miscellaneous endpoints used across the site.

use std::collections::HashMap;

use anyhow::Result;
use askama::Template;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use chrono::Duration;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    db::DynDB,
    handlers::{error::HandlerError, prepare_headers},
    templates::{
        PageId,
        auth::User,
        misc::{self, UserMenuSection},
    },
};

use super::auth::AUTH_PROVIDER_KEY;

/// Handler for rendering the not found (404) page.
#[instrument(skip_all, err)]
pub(crate) async fn not_found(
    State(cfg): State<HttpServerConfig>,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = misc::NotFoundPage {
        auth_provider: None,
        cfg: cfg.into(),
        page_id: PageId::NotFound,
        user: User::default(),
    };

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Html(template.render()?)).into_response())
}

/// Handler for searching locations and returning results as JSON.
#[instrument(skip_all, err)]
pub(crate) async fn search_locations(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get locations from the database
    let Some(ts_query) = query.get("ts_query") else {
        return Ok((StatusCode::BAD_REQUEST, "missing ts_query parameter").into_response());
    };
    let locations = db.search_locations(ts_query).await?;

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Json(locations)).into_response())
}

/// Handler for searching members and returning results as JSON.
#[instrument(skip_all, err)]
pub(crate) async fn search_members(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get members from the database
    let (Some(foundation), Some(member)) = (query.get("foundation"), query.get("member")) else {
        return Ok((StatusCode::BAD_REQUEST, "missing foundation or member parameter").into_response());
    };
    let members = db.search_members(foundation, member).await?;

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Json(members)).into_response())
}

/// Handler for searching projects and returning results as JSON.
#[instrument(skip_all, err)]
pub(crate) async fn search_projects(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get projects from the database
    let (Some(foundation), Some(project)) = (query.get("foundation"), query.get("project")) else {
        return Ok((StatusCode::BAD_REQUEST, "missing foundation or project parameter").into_response());
    };
    let projects = db.search_projects(foundation, project).await?;

    // Prepare response headers
    let headers = prepare_headers(Duration::hours(1), &[])?;

    Ok((headers, Json(projects)).into_response())
}

/// Handler for rendering the user menu section in the header.
#[instrument(skip_all, err)]
pub(crate) async fn user_menu_section(
    auth_session: AuthSession,
    session: Session,
) -> Result<impl IntoResponse, HandlerError> {
    // Prepare template
    let template = UserMenuSection {
        user: auth_session.into(),
        auth_provider: session.get(AUTH_PROVIDER_KEY).await?,
    };

    Ok(Html(template.render()?))
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
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::mock::MockDB,
        handlers::tests::{
            TestRouterBuilder, sample_auth_user, sample_location, sample_session_record, with_auth_provider,
        },
        notifications::MockNotificationsManager,
    };

    #[tokio::test]
    async fn test_not_found_returns_html_with_cache_headers() {
        // Setup router and send request to unknown path
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/missing-path")
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
    async fn test_search_locations_returns_bad_request_when_ts_query_is_missing() {
        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/locations/search")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(bytes.as_ref(), b"missing ts_query parameter");
    }

    #[tokio::test]
    async fn test_search_locations_returns_json_results() {
        // Setup database mock
        let mut db = MockDB::new();
        db.expect_search_locations()
            .times(1)
            .withf(|ts_query| ts_query == "san")
            .returning(|_| Ok(vec![sample_location()]));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/locations/search?ts_query=san")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CACHE_CONTROL], "max-age=0");
        assert_eq!(parts.headers[CONTENT_TYPE], "application/json");
        assert!(!bytes.is_empty());
    }

    #[tokio::test]
    async fn test_search_members_returns_bad_request_when_params_are_missing() {
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

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/dashboard/members/search?foundation=CNCF")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(bytes.as_ref(), b"missing foundation or member parameter");
    }

    #[tokio::test]
    async fn test_search_members_returns_json_results() {
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
        db.expect_search_members()
            .times(1)
            .withf(|foundation, member| foundation == "CNCF" && member == "example")
            .returning(|_, _| Ok(Vec::new()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/dashboard/members/search?foundation=CNCF&member=example")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_search_projects_returns_bad_request_when_params_are_missing() {
        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/projects/search?project=example")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(bytes.as_ref(), b"missing foundation or project parameter");
    }

    #[tokio::test]
    async fn test_search_projects_returns_json_results() {
        // Setup database mock
        let mut db = MockDB::new();
        db.expect_search_projects()
            .times(1)
            .withf(|foundation, project| foundation == "CNCF" && project == "kubernetes")
            .returning(|_, _| Ok(Vec::new()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/projects/search?foundation=CNCF&project=kubernetes")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_user_menu_section_renders_authenticated_user() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let mut session_record = sample_session_record(session_id, user_id, auth_hash, None);
        with_auth_provider(&mut session_record, "github");

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

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/section/user-menu")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CONTENT_TYPE], "text/html; charset=utf-8");
        assert!(!bytes.is_empty());
    }
}
