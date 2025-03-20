//! This module defines some HTTP handlers used across the site.

use std::collections::HashMap;

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tracing::instrument;

use crate::{db::DynDB, handlers::error::HandlerError, templates::misc};

/// Handler that returns the not found page.
#[instrument(skip_all, err)]
pub(crate) async fn not_found() -> Result<impl IntoResponse, HandlerError> {
    let template = misc::NotFound {};
    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the locations search results.
#[instrument(skip_all, err)]
pub(crate) async fn search_locations(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    let Some(ts_query) = query.get("ts_query_location") else {
        return Ok((StatusCode::BAD_REQUEST, "missing ts_query parameter").into_response());
    };
    let locations = db.search_locations(ts_query).await?;
    let template = misc::Locations { locations };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the members search results.
#[instrument(skip_all, err)]
pub(crate) async fn search_members(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    let Some(name) = query.get("name") else {
        return Ok((StatusCode::BAD_REQUEST, "missing name parameter").into_response());
    };
    let members = db.search_members(name).await?;
    let template = misc::Members { members };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the projects search results.
#[instrument(skip_all, err)]
pub(crate) async fn search_projects(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    let Some(name) = query.get("name") else {
        return Ok((StatusCode::BAD_REQUEST, "missing name parameter").into_response());
    };
    let projects = db.search_projects(name).await?;
    let template = misc::Projects { projects };

    Ok(Html(template.render()?).into_response())
}
