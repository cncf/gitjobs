//! HTTP handlers for image management, including upload and retrieval.

use std::{borrow::Cow, io::Cursor, str::FromStr};

use anyhow::{Result, anyhow};
use axum::{
    extract::{Multipart, Path, State},
    http::{
        HeaderMap, HeaderValue, StatusCode, Uri,
        header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, REFERER},
    },
    response::IntoResponse,
};
use image::{ImageFormat as DetectedImageFormat, ImageReader};
use quick_xml::{Reader, events::Event};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    config::HttpServerConfig,
    handlers::error::HandlerError,
    img::{DynImageStore, ImageFormat},
};

/// Cache-Control header for immutable responses.
const CACHE_CONTROL_IMMUTABLE: &str = "max-age=2592000, immutable";

/// Maximum payload size allowed for image uploads (1 MiB).
const MAX_IMAGE_SIZE_BYTES: usize = 1024 * 1024;

// Handlers

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
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL_IMMUTABLE));
    headers.insert(CONTENT_LENGTH, data.len().into());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

    Ok((headers, data).into_response())
}

/// Handles image upload from authenticated users, saving the image to the store.
#[instrument(skip_all, err)]
pub(crate) async fn upload(
    auth_session: AuthSession,
    State(server_cfg): State<HttpServerConfig>,
    State(image_store): State<DynImageStore>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Validate referer header matches configured hostname
    if !referer_matches_site(&server_cfg, &headers)? {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    // Extract target, file name and bytes from multipart payload
    let mut target: Option<ImageTarget> = None;
    let mut file_name: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("target") => {
                let Ok(target_value) = field.text().await else {
                    return Ok((StatusCode::BAD_REQUEST).into_response());
                };
                target = match ImageTarget::from_str(&target_value) {
                    Ok(target) => Some(target),
                    Err(err) => {
                        return Ok((StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response());
                    }
                };
            }
            Some("file" | "logo") => {
                file_name = field.file_name().map(str::to_string);
                let Ok(bytes) = field.bytes().await else {
                    return Ok((StatusCode::BAD_REQUEST).into_response());
                };
                data = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    // Ensure multipart payload includes required fields
    let Some(file_name) = file_name else {
        return Ok((StatusCode::BAD_REQUEST, "missing file in upload payload").into_response());
    };
    let Some(data) = data else {
        return Ok((StatusCode::BAD_REQUEST, "missing file in upload payload").into_response());
    };
    let Some(target) = target else {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            "missing target in upload payload",
        )
            .into_response());
    };

    // Enforce maximum file size
    if data.len() > MAX_IMAGE_SIZE_BYTES {
        return Ok((StatusCode::PAYLOAD_TOO_LARGE, "image exceeds 1MB limit").into_response());
    }

    // Detect image format and check extension matches
    let extension = match image_extension(&file_name) {
        Ok(extension) => extension,
        Err(err) => {
            return Ok((StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response());
        }
    };
    let format = match detect_image_format(&data, extension.as_ref()) {
        Ok(format) => format,
        Err(err) => {
            return Ok((StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response());
        }
    };
    if !extension_matches(&format, extension.as_ref()) {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            "file extension does not match detected image format",
        )
            .into_response());
    }

    // Validate target dimensions for non-SVG uploads
    if !matches!(format, SupportedImageFormat::Svg)
        && let Err(err) = validate_image_dimensions(&data, target)
    {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response());
    }

    // Save image to store
    let image_id = image_store.save(&user.user_id, &file_name, data).await?;

    Ok((StatusCode::OK, image_id.to_string()).into_response())
}

// Helpers

/// Detects the image format using the `image` crate with a fallback for SVGs.
fn detect_image_format(bytes: &[u8], extension: &str) -> Result<SupportedImageFormat> {
    match image::guess_format(bytes) {
        Ok(DetectedImageFormat::Gif) => Ok(SupportedImageFormat::Gif),
        Ok(DetectedImageFormat::Jpeg) => Ok(SupportedImageFormat::Jpeg),
        Ok(DetectedImageFormat::Png) => Ok(SupportedImageFormat::Png),
        Ok(DetectedImageFormat::Tiff) => Ok(SupportedImageFormat::Tiff),
        Ok(DetectedImageFormat::WebP) => Ok(SupportedImageFormat::Webp),
        Ok(other) => Err(anyhow!("unsupported image format: {other:?}")),
        Err(_) if is_svg(bytes, extension) => Ok(SupportedImageFormat::Svg),
        Err(_) => Err(anyhow!("unsupported image format")),
    }
}

/// Returns the accepted extensions for the provided format.
fn expected_extensions(format: &SupportedImageFormat) -> &'static [&'static str] {
    match format {
        SupportedImageFormat::Gif => &["gif"],
        SupportedImageFormat::Jpeg => &["jpg", "jpeg"],
        SupportedImageFormat::Png => &["png"],
        SupportedImageFormat::Svg => &["svg"],
        SupportedImageFormat::Tiff => &["tif", "tiff"],
        SupportedImageFormat::Webp => &["webp"],
    }
}

/// Validates that the extension matches the detected image format.
fn extension_matches(format: &SupportedImageFormat, extension: &str) -> bool {
    expected_extensions(format)
        .iter()
        .any(|candidate| candidate == &extension)
}

/// Extracts the lowercase file extension from a file name.
fn image_extension(file_name: &str) -> Result<Cow<'_, str>> {
    let extension = file_name
        .rsplit('.')
        .next()
        .ok_or_else(|| anyhow!("missing file extension"))?;
    if extension.is_empty() {
        return Err(anyhow!("missing file extension"));
    }
    Ok(Cow::from(extension.to_ascii_lowercase()))
}

/// Determines whether the provided bytes and extension represent a valid SVG asset.
fn is_svg(bytes: &[u8], extension: &str) -> bool {
    const SVG_NAMESPACE: &[u8] = b"http://www.w3.org/2000/svg";
    const DANGEROUS_ELEMENTS: &[&[u8]] = &[b"script", b"foreignObject"];

    // Check extension first for a fast reject path
    if !extension.eq_ignore_ascii_case("svg") {
        return false;
    }

    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut found_svg_root = false;
    let mut in_root = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref event) | Event::Empty(ref event)) => {
                let tag_name = event.name();

                if !in_root {
                    if tag_name.as_ref() != b"svg" {
                        return false;
                    }

                    let has_svg_namespace = event.attributes().filter_map(Result::ok).any(|attr| {
                        (attr.key.as_ref() == b"xmlns" || attr.key.local_name().as_ref() == b"xmlns")
                            && attr.value.as_ref() == SVG_NAMESPACE
                    });

                    if !has_svg_namespace {
                        return false;
                    }

                    found_svg_root = true;
                    in_root = true;
                }

                for dangerous in DANGEROUS_ELEMENTS {
                    if tag_name.as_ref().eq_ignore_ascii_case(dangerous) {
                        return false;
                    }
                }

                for attr in event.attributes().filter_map(Result::ok) {
                    let key = attr.key.as_ref();
                    let value = attr.value.as_ref();

                    if key.len() >= 2 && key[..2].eq_ignore_ascii_case(b"on") {
                        return false;
                    }

                    if (key == b"href" || key == b"xlink:href")
                        && value.len() >= 11
                        && value[..11].eq_ignore_ascii_case(b"javascript:")
                    {
                        return false;
                    }

                    if (key == b"href" || key == b"xlink:href")
                        && value.len() >= 5
                        && value[..5].eq_ignore_ascii_case(b"data:")
                        && (value.len() < 11 || !value[5..11].eq_ignore_ascii_case(b"image/"))
                    {
                        return false;
                    }
                }
            }
            Ok(_) => {}
            Err(_) => return false,
        }
        buf.clear();
    }

    found_svg_root
}

/// Checks whether the referer header matches the configured site hostname.
fn referer_matches_site(server_cfg: &HttpServerConfig, headers: &HeaderMap) -> Result<bool> {
    if server_cfg.disable_referer_checks {
        return Ok(true);
    }

    let Some(referer) = headers.get(REFERER) else {
        return Ok(false);
    };
    let Ok(referer) = referer.to_str() else {
        return Ok(false);
    };

    let referer_host = Uri::from_str(referer)
        .ok()
        .and_then(|uri| uri.host().map(str::to_ascii_lowercase));
    let site_host = Uri::from_str(&server_cfg.base_url)
        .ok()
        .and_then(|uri| uri.host().map(str::to_ascii_lowercase))
        .ok_or_else(|| anyhow!("missing host in base_url"))?;

    Ok(referer_host.is_some_and(|referer_host| referer_host == site_host))
}

/// Validates image dimensions match the target requirements.
fn validate_image_dimensions(bytes: &[u8], target: ImageTarget) -> Result<()> {
    let (expected_width, expected_height) = target.dimensions();
    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let (width, height) = reader.into_dimensions()?;

    if width != expected_width || height != expected_height {
        return Err(anyhow!(
            "image dimensions {width}x{height} do not match required {expected_width}x{expected_height}"
        ));
    }

    Ok(())
}

// Types

/// Image target defining expected dimensions.
#[derive(Clone, Copy, Debug)]
enum ImageTarget {
    Logo,
    Photo,
}

impl ImageTarget {
    /// Returns (width, height) for the target.
    fn dimensions(self) -> (u32, u32) {
        match self {
            ImageTarget::Logo | ImageTarget::Photo => (400, 400),
        }
    }
}

impl FromStr for ImageTarget {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "logo" => Ok(ImageTarget::Logo),
            "photo" => Ok(ImageTarget::Photo),
            _ => Err(anyhow!("unknown image target: {value}")),
        }
    }
}

/// Supported image formats accepted by the upload endpoint.
enum SupportedImageFormat {
    Gif,
    Jpeg,
    Png,
    Svg,
    Tiff,
    Webp,
}

// Tests

#[cfg(test)]
mod tests {
    use std::{io::Cursor, str::FromStr};

    use axum::{
        body::{Body, to_bytes},
        http::{
            HeaderMap, HeaderValue, Request, StatusCode,
            header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, REFERER},
        },
    };
    use axum_login::tower_sessions::session;
    use image::{DynamicImage, ImageFormat as EncodedImageFormat, Rgba, RgbaImage};
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::mock::MockDB,
        handlers::tests::{
            TestRouterBuilder, sample_auth_user, sample_image, sample_session_record, test_http_server_cfg,
        },
        img::{ImageFormat, MockImageStore},
        notifications::MockNotificationsManager,
    };

    use super::*;

    const PNG_1X1_BYTES: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00,
        0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01,
        0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    const SVG_BYTES_SAFE: &[u8] =
        br#"<svg xmlns="http://www.w3.org/2000/svg"><circle cx="50" cy="50" r="40"/></svg>"#;
    const SVG_BYTES_UNSAFE: &[u8] =
        br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#;

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
        assert_eq!(parts.headers[CACHE_CONTROL], CACHE_CONTROL_IMMUTABLE);
        assert_eq!(parts.headers[CONTENT_TYPE], "image/png");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_is_svg_accepts_valid_svg() {
        assert!(is_svg(SVG_BYTES_SAFE, "svg"));
    }

    #[test]
    fn test_is_svg_rejects_script_element() {
        assert!(!is_svg(SVG_BYTES_UNSAFE, "svg"));
    }

    #[test]
    fn test_referer_matches_site_returns_false_for_mismatch() {
        let cfg = test_http_server_cfg();
        let mut headers = HeaderMap::new();
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://unauthorized.test/dashboard"),
        );

        let matches = referer_matches_site(&cfg, &headers).unwrap();
        assert!(!matches);
    }

    #[test]
    fn test_referer_matches_site_returns_true_when_checks_disabled() {
        let mut cfg = test_http_server_cfg();
        cfg.disable_referer_checks = true;

        let matches = referer_matches_site(&cfg, &HeaderMap::new()).unwrap();
        assert!(matches);
    }

    #[tokio::test]
    async fn test_upload_allows_missing_referer_when_checks_disabled() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let image_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(
            boundary,
            "avatar.svg",
            "image/svg+xml",
            SVG_BYTES_SAFE,
            Some("logo"),
        );

        // Setup database mock
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());

        // Setup image store mock
        let mut image_store = MockImageStore::new();
        image_store
            .expect_save()
            .times(1)
            .withf(move |id, filename, data| {
                *id == user_id && filename == "avatar.svg" && data == SVG_BYTES_SAFE
            })
            .returning(move |_, _, _| Box::pin(async move { Ok(image_id) }));

        // Setup router and send request
        let mut cfg = test_http_server_cfg();
        cfg.disable_referer_checks = true;
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .with_image_store(image_store)
            .build()
            .await;
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

    #[tokio::test]
    async fn test_upload_rejects_extension_mismatch() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(
            boundary,
            "avatar.svg",
            "image/png",
            &png_bytes(400, 400),
            Some("logo"),
        );

        // Setup database and image store mocks
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());
        let mut image_store = MockImageStore::new();
        image_store.expect_save().never();

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_upload_rejects_missing_referer() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(
            boundary,
            "avatar.svg",
            "image/svg+xml",
            SVG_BYTES_SAFE,
            Some("logo"),
        );

        // Setup database and image store mocks
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());
        let mut image_store = MockImageStore::new();
        image_store.expect_save().never();

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_upload_rejects_unsafe_svg() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(
            boundary,
            "avatar.svg",
            "image/svg+xml",
            SVG_BYTES_UNSAFE,
            Some("logo"),
        );

        // Setup database and image store mocks
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());
        let mut image_store = MockImageStore::new();
        image_store.expect_save().never();

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_upload_rejects_unknown_target() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(
            boundary,
            "avatar.svg",
            "image/svg+xml",
            SVG_BYTES_SAFE,
            Some("unknown"),
        );

        // Setup database and image store mocks
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());
        let mut image_store = MockImageStore::new();
        image_store.expect_save().never();

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
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
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record);

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let boundary = "X-BOUNDARY";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"target\"\r\n\r\n\
             logo\r\n\
             --{boundary}--\r\n"
        );
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
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
        let valid_png = png_bytes(400, 400);

        // Setup database mock
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());

        // Setup image store mock
        let valid_png_for_mock = valid_png.clone();
        let mut image_store = MockImageStore::new();
        image_store
            .expect_save()
            .times(1)
            .withf(move |id, filename, data| {
                *id == user_id && filename == "avatar.png" && data == &valid_png_for_mock
            })
            .returning(move |_, _, _| Box::pin(async move { Ok(image_id) }));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(boundary, "avatar.png", "image/png", &valid_png, Some("logo"));
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        // Check response matches expectations
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(bytes.as_ref(), image_id.to_string().as_bytes());
    }

    #[tokio::test]
    async fn test_upload_returns_payload_too_large_when_file_exceeds_limit() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let oversized = vec![0_u8; MAX_IMAGE_SIZE_BYTES + 1];
        let body = build_multipart_body(boundary, "avatar.png", "image/png", &oversized, Some("logo"));

        // Setup database and image store mocks
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());
        let mut image_store = MockImageStore::new();
        image_store.expect_save().never();

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_upload_returns_unprocessable_entity_when_dimensions_do_not_match() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(boundary, "avatar.png", "image/png", PNG_1X1_BYTES, Some("photo"));

        // Setup database and image store mocks
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());
        let mut image_store = MockImageStore::new();
        image_store.expect_save().never();

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_upload_returns_unprocessable_entity_when_target_is_missing() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let boundary = "X-BOUNDARY";
        let body = build_multipart_body(boundary, "avatar.svg", "image/svg+xml", SVG_BYTES_SAFE, None);

        // Setup database and image store mocks
        let mut db = MockDB::new();
        setup_authenticated_user(&mut db, auth_hash, session_id, user_id, session_record.clone());
        let mut image_store = MockImageStore::new();
        image_store.expect_save().never();

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_image_store(image_store)
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/images")
            .header(COOKIE, format!("id={session_id}"))
            .header(CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
            .header(REFERER, "http://localhost:9000/dashboard")
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_validate_image_dimensions_rejects_wrong_dimensions() {
        let result = validate_image_dimensions(PNG_1X1_BYTES, ImageTarget::Logo);
        assert_eq!(
            result.unwrap_err().to_string(),
            "image dimensions 1x1 do not match required 400x400"
        );
    }

    #[test]
    fn test_image_target_from_str_accepts_logo() {
        let target = ImageTarget::from_str("logo").unwrap();
        assert!(matches!(target, ImageTarget::Logo));
    }

    // Helpers

    fn build_multipart_body(
        boundary: &str,
        file_name: &str,
        content_type: &str,
        bytes: &[u8],
        target: Option<&str>,
    ) -> Vec<u8> {
        let mut body = Vec::new();

        if let Some(target) = target {
            body.extend_from_slice(
                format!(
                    "--{boundary}\r\n\
                     Content-Disposition: form-data; name=\"target\"\r\n\r\n\
                     {target}\r\n"
                )
                .as_bytes(),
            );
        }

        body.extend_from_slice(
            format!(
                "--{boundary}\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
                 Content-Type: {content_type}\r\n\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        body
    }

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let img = RgbaImage::from_pixel(width, height, Rgba([12, 34, 56, 255]));
        let mut bytes = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut bytes), EncodedImageFormat::Png)
            .unwrap();
        bytes
    }

    fn setup_authenticated_user(
        db: &mut MockDB,
        auth_hash: &'static str,
        session_id: session::Id,
        user_id: Uuid,
        session_record: session::Record,
    ) {
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
    }
}
