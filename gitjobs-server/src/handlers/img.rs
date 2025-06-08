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

/// Maximum allowed file size for image uploads (2MB)
const MAX_FILE_SIZE: usize = 2 * 1024 * 1024;

/// Supported image file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "tiff", "svg"];

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
        tracing::warn!("Image upload attempted without authentication");
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    tracing::info!("Processing image upload for user: {}", user.user_id);

    // Get image file name and data from the multipart form data
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => {
            tracing::error!("No multipart field found in upload request");
            return Ok((StatusCode::BAD_REQUEST, "No file provided").into_response());
        }
        Err(e) => {
            tracing::error!("Failed to read multipart field: {}", e);
            // This often happens when the request exceeds size limits
            return Ok((StatusCode::PAYLOAD_TOO_LARGE, format!("File too large. Maximum size allowed is {}MB", MAX_FILE_SIZE / (1024 * 1024))).into_response());
        }
    };

    let file_name = field.file_name().unwrap_or("unknown").to_string();
    tracing::info!("Uploading file: {}", file_name);

    // Validate file extension
    if let Some(extension) = file_name.split('.').last() {
        let ext_lower = extension.to_lowercase();
        if !SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
            tracing::warn!("Unsupported file extension: {}", extension);
            return Ok((StatusCode::BAD_REQUEST, format!("Unsupported file type. Supported formats: {}", SUPPORTED_EXTENSIONS.join(", "))).into_response());
        }
    } else {
        tracing::warn!("File has no extension: {}", file_name);
        return Ok((StatusCode::BAD_REQUEST, "File must have a valid extension").into_response());
    }

    let data = match field.bytes().await {
        Ok(data) => {
            if data.is_empty() {
                tracing::error!("Uploaded file is empty");
                return Ok((StatusCode::BAD_REQUEST, "File is empty").into_response());
            }
            
            if data.len() > MAX_FILE_SIZE {
                tracing::warn!("File size {} bytes exceeds limit of {} bytes", data.len(), MAX_FILE_SIZE);
                return Ok((StatusCode::PAYLOAD_TOO_LARGE, format!("File size {}MB exceeds maximum allowed size of {}MB", data.len() / (1024 * 1024), MAX_FILE_SIZE / (1024 * 1024))).into_response());
            }
            
            tracing::info!("File size: {} bytes", data.len());
            data
        }
        Err(e) => {
            tracing::error!("Failed to read file bytes: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "Failed to read file data").into_response());
        }
    };

    // Save image to store
    match image_store.save(&user.user_id, &file_name, data.to_vec()).await {
        Ok(image_id) => {
            tracing::info!("Image saved successfully with ID: {}", image_id);
            Ok((StatusCode::OK, image_id.to_string()).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to save image: {}", e);
            Ok((StatusCode::INTERNAL_SERVER_ERROR, "Failed to process image").into_response())
        }
    }
}
