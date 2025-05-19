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
