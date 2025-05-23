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
