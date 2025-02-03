//! This module defines the router used to dispatch HTTP requests to the
//! corresponding handler.

use anyhow::Result;
use axum::{
    extract::FromRef,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        HeaderValue, StatusCode, Uri,
    },
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use axum_login::login_required;
use axum_messages::MessagesManagerLayer;
use rust_embed::Embed;
use tower::ServiceBuilder;
use tower_http::{
    set_header::SetResponseHeaderLayer, trace::TraceLayer, validate_request::ValidateRequestHeaderLayer,
};
use tracing::instrument;

use crate::{
    auth::AuthnBackend,
    config::HttpServerConfig,
    db::DynDB,
    handlers::{
        auth::{self, LOG_IN_URL},
        common, dashboard, jobboard,
        middleware::{user_owns_employer, user_owns_job},
    },
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
pub(crate) fn setup(cfg: &HttpServerConfig, db: DynDB) -> Result<Router> {
    // Setup router state
    let state = State { db: db.clone() };

    // Setup auth layer and middleware
    let auth_layer = crate::auth::setup_layer(cfg, db)?;
    let check_user_owns_employer = middleware::from_fn_with_state(state.clone(), user_owns_employer);
    let check_user_owns_job = middleware::from_fn_with_state(state.clone(), user_owns_job);

    // Setup router
    let mut router = Router::new()
        .route("/dashboard", get(dashboard::home::page))
        .route(
            "/dashboard/employers/add",
            get(dashboard::employers::add_page).post(dashboard::employers::add),
        )
        .route(
            "/dashboard/employers/update",
            get(dashboard::employers::update_page).put(dashboard::employers::update),
        )
        .route(
            "/dashboard/employers/{:employer_id}/select",
            put(dashboard::employers::select).layer(check_user_owns_employer.clone()),
        )
        .route("/dashboard/jobs/list", get(dashboard::jobs::list_page))
        .route(
            "/dashboard/jobs/add",
            get(dashboard::jobs::add_page).post(dashboard::jobs::add),
        )
        .route("/dashboard/jobs/preview", post(dashboard::jobs::preview_page))
        .route(
            "/dashboard/jobs/{:job_id}/archive",
            put(dashboard::jobs::archive).layer(check_user_owns_job.clone()),
        )
        .route(
            "/dashboard/jobs/{:job_id}/delete",
            delete(dashboard::jobs::delete).layer(check_user_owns_job.clone()),
        )
        .route(
            "/dashboard/jobs/{:job_id}/publish",
            put(dashboard::jobs::publish).layer(check_user_owns_job.clone()),
        )
        .route(
            "/dashboard/jobs/{:job_id}/update",
            get(dashboard::jobs::update_page)
                .layer(check_user_owns_job.clone())
                .put(dashboard::jobs::update)
                .layer(check_user_owns_job.clone()),
        )
        .route("/locations/search", get(common::search_locations))
        .route_layer(login_required!(
            AuthnBackend,
            login_url = LOG_IN_URL,
            redirect_field = "next_url"
        ))
        .route("/", get(jobboard::jobs::page))
        .route("/about", get(jobboard::about::page))
        .route("/health-check", get(health_check))
        .route("/jobs", get(jobboard::jobs::page))
        .route("/log-in", get(auth::log_in_page).post(auth::log_in))
        .route("/log-in/oauth2/{:provider}", get(auth::oauth2_redirect))
        .route("/log-in/oauth2/{:provider}/callback", get(auth::oauth2_callback))
        .route("/log-out", get(auth::log_out))
        .route("/sign-up", get(auth::sign_up_page).post(auth::sign_up))
        .route("/static/{*file}", get(static_handler))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::try_from(format!("max-age={DEFAULT_CACHE_DURATION}")).expect("valid header value"),
        ))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(MessagesManagerLayer)
        .layer(auth_layer)
        .with_state(state);

    // Setup basic auth
    if let Some(basic_auth) = &cfg.basic_auth {
        if basic_auth.enabled {
            router = router.layer(ValidateRequestHeaderLayer::basic(
                &basic_auth.username,
                &basic_auth.password,
            ));
        }
    }

    Ok(router)
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
