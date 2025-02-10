//! This module defines some HTTP handlers used across the site.

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{db::DynDB, handlers::error::HandlerError, img::DynImageStore, templates::common};

/// Handler that returns the locations search results.
#[instrument(skip_all, err)]
pub(crate) async fn search_locations(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    let Some(ts_query) = query.get("ts_query") else {
        return Ok((StatusCode::BAD_REQUEST, "missing ts_query parameter").into_response());
    };
    let locations = db.search_locations(ts_query).await?;
    let template = common::Locations { locations };

    Ok(Html(template.render()?).into_response())
}

/// Handler that uploads an image.
#[instrument(skip_all, err)]
pub(crate) async fn upload_image(
    State(image_store): State<DynImageStore>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HandlerError> {
    // Get image file name and data
    let (file_name, data) = if let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or_default().to_string();
        let Ok(data) = field.bytes().await else {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        };
        (file_name, data)
    } else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };

    // Save image to store
    let image_id = image_store.save(&file_name, data.to_vec()).await?;

    Ok((StatusCode::OK, image_id.to_string()).into_response())
}
