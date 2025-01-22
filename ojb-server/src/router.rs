//! This module defines the router used to dispatch HTTP requests to the
//! corresponding handler.

use axum::{
    extract::FromRef,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        HeaderValue, StatusCode, Uri,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::Embed;
use tower::ServiceBuilder;
use tower_http::{
    set_header::SetResponseHeaderLayer, trace::TraceLayer, validate_request::ValidateRequestHeaderLayer,
};
use tracing::instrument;

use crate::{
    config::HttpServerConfig,
    db::DynDB,
    handlers::{common, dashboard, jobboard},
};

/// Default cache duration.
#[cfg(debug_assertions)]
pub(crate) const DEFAULT_CACHE_DURATION: usize = 0; // No cache
#[cfg(not(debug_assertions))]
pub(crate) const DEFAULT_CACHE_DURATION: usize = 60 * 5; // 5 minutes

/// Embed static files in the binary.
#[derive(Embed)]
#[folder = "static"]
struct StaticFile;

/// Router's state.
#[derive(Clone, FromRef)]
pub(crate) struct State {
    pub db: DynDB,
}

/// Setup router.
#[instrument(skip_all)]
pub(crate) fn setup(cfg: &HttpServerConfig, db: DynDB) -> Router {
    // Setup router
    let mut router = Router::new()
        .route("/", get(jobboard::jobs::page))
        .route("/jobs", get(jobboard::jobs::page))
        .route("/about", get(jobboard::about::page))
        .route("/dashboard", get(dashboard::page))
        .route("/dashboard/jobs", get(dashboard::jobs::list_page))
        .route(
            "/dashboard/jobs/add",
            get(dashboard::jobs::add_page).post(dashboard::jobs::add_job),
        )
        .route("/dashboard/settings", get(dashboard::settings::page))
        .route("/locations/search", get(common::search_locations))
        .route("/health-check", get(health_check))
        .route("/static/{*file}", get(static_handler))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::try_from(format!("max-age={DEFAULT_CACHE_DURATION}")).expect("valid header value"),
        ))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(State { db });

    // Setup basic auth
    if let Some(basic_auth) = &cfg.basic_auth {
        if basic_auth.enabled {
            router = router.layer(ValidateRequestHeaderLayer::basic(
                &basic_auth.username,
                &basic_auth.password,
            ));
        }
    }

    router
}

/// Handler that takes care of health check requests.
#[instrument(skip_all)]
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

/// Handler that serves static files.
#[instrument]
async fn static_handler(uri: Uri) -> impl IntoResponse {
    // Extract file path from URI
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.starts_with("static/") {
        path = path.replace("static/", "");
    }

    // Set cache duration based on resource type
    #[cfg(not(debug_assertions))]
    let cache_max_age = if path.starts_with("images/") {
        60 * 60 * 24 * 7 // 1 week
    } else {
        // Default cache duration for other static resources
        60 * 60 // 1 hour
    };
    #[cfg(debug_assertions)]
    let cache_max_age = 0;

    // Get file content and return it (if available)
    match StaticFile::get(path.as_str()) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let cache = format!("max-age={cache_max_age}");
            let headers = [(CONTENT_TYPE, mime.as_ref()), (CACHE_CONTROL, &cache)];
            (headers, file.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
