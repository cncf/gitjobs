//! HTTP handlers for image management, including upload and retrieval.

use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use reqwest::{
    StatusCode,
    header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE},
};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    handlers::error::HandlerError,
    img::{DynImageStore, ImageFormat},
};

/// Returns an image from the store, setting headers for cache and content type.
#[instrument(skip_all, err)]
pub(crate) async fn get(
    State(image_store): State<DynImageStore>,
    Path((image_id, version)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get image from the store
    let Some((data, format)) = image_store.get(image_id, &version).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    // Prepare response headers
    let mut headers = HeaderMap::new();
    let content_type = match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Svg => "image/svg+xml",
    };
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("max-age=2592000, immutable"),
    );
    headers.insert(CONTENT_LENGTH, data.len().into());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

    Ok((headers, data).into_response())
}

/// Handles image upload from authenticated users, saving the image to the store.
#[instrument(skip_all, err)]
pub(crate) async fn upload(
    auth_session: AuthSession,
    State(image_store): State<DynImageStore>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Get image file name and data from the multipart form data
    let (file_name, data) = if let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or_default().to_string();
        let Ok(data) = field.bytes().await else {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        };
        (file_name, data)
    } else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    // Save image to store
    let image_id = image_store.save(&user.user_id, &file_name, data.to_vec()).await?;

    Ok((StatusCode::OK, image_id.to_string()).into_response())
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
        handlers::tests::{TestRouterBuilder, sample_auth_user, sample_image, sample_session_record},
        img::{ImageFormat, MockImageStore},
        notifications::MockNotificationsManager,
    };

    #[tokio::test]
    async fn test_get_public_image_returns_not_found_when_missing() {
        // Setup identifiers and data structures
        let image_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_is_image_public()
            .times(1)
            .withf(move |id| *id == image_id)
            .returning(|_| Ok(true));

        // Setup image store mock
        let mut image_store = MockImageStore::new();
        image_store
            .expect_get()
            .times(1)
            .withf(move |id, version| *id == image_id && version == "small")
            .returning(|_, _| Box::pin(async { Ok(None) }));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/jobboard/images/{image_id}/small"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_public_image_returns_png_with_headers() {
        // Setup identifiers and data structures
        let image_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_is_image_public()
            .times(1)
            .withf(move |id| *id == image_id)
            .returning(|_| Ok(true));

        // Setup image store mock
        let mut image_store = MockImageStore::new();
        image_store
            .expect_get()
            .times(1)
            .withf(move |id, version| *id == image_id && version == "small")
            .returning(|_, _| Box::pin(async { Ok(Some(sample_image())) }));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/jobboard/images/{image_id}/small"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CACHE_CONTROL], "max-age=2592000, immutable");
        assert_eq!(parts.headers[CONTENT_TYPE], "image/png");
        assert!(!bytes.is_empty());
    }

    #[tokio::test]
    async fn test_get_public_image_returns_forbidden_when_not_public() {
        // Setup identifiers and data structures
        let image_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_is_image_public()
            .times(1)
            .withf(move |id| *id == image_id)
            .returning(|_| Ok(false));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/jobboard/images/{image_id}/small"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_dashboard_image_returns_svg_for_authorized_user() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let image_id = Uuid::new_v4();
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
        db.expect_user_has_image_access()
            .times(1)
            .withf(move |user, image| *user == user_id && *image == image_id)
            .returning(|_, _| Ok(true));

        // Setup image store mock
        let mut image_store = MockImageStore::new();
        image_store
            .expect_get()
            .times(1)
            .withf(move |id, version| *id == image_id && version == "svg")
            .returning(|_, _| Box::pin(async { Ok(Some((b"<svg></svg>".to_vec(), ImageFormat::Svg))) }));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/dashboard/images/{image_id}/svg"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(parts.headers[CONTENT_TYPE], "image/svg+xml");
        assert!(!bytes.is_empty());
    }

    #[tokio::test]
    async fn test_upload_returns_bad_request_when_no_file_is_sent() {
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
        let boundary = "X-BOUNDARY";
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(format!("--{boundary}--\r\n")))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_upload_returns_image_id_when_file_is_saved() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let image_id = Uuid::new_v4();
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

        // Setup image store mock
        let mut image_store = MockImageStore::new();
        image_store
            .expect_save()
            .times(1)
            .withf(move |id, filename, data| *id == user_id && filename == "avatar.png" && data == b"PNGDATA")
            .returning(move |_, _, _| Box::pin(async move { Ok(image_id) }));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let boundary = "X-BOUNDARY";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"avatar.png\"\r\n\
             Content-Type: image/png\r\n\r\n\
             PNGDATA\r\n\
             --{boundary}--\r\n"
        );
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(bytes.as_ref(), image_id.to_string().as_bytes());
    }
}
