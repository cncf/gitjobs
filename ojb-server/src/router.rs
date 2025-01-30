//! This module defines the router used to dispatch HTTP requests to the
//! corresponding handler.

use axum::{
    extract::FromRef,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        HeaderValue, StatusCode, Uri,
    },
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use axum_login::{
    login_required,
    tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer},
    AuthManagerLayer, AuthManagerLayerBuilder,
};
use axum_messages::MessagesManagerLayer;
use rust_embed::Embed;
use time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    set_header::SetResponseHeaderLayer, trace::TraceLayer, validate_request::ValidateRequestHeaderLayer,
};
use tower_sessions::CachingSessionStore;
use tower_sessions_moka_store::MokaStore;
use tracing::instrument;

use crate::{
    auth::{AuthnBackend, SessionStore},
    config::HttpServerConfig,
    db::DynDB,
    handlers::{
        auth::{self, LOG_IN_URL},
        common, dashboard, jobboard,
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
pub(crate) fn setup(cfg: &HttpServerConfig, db: DynDB) -> Router {
    // Setup authentication/authorization layer
    let auth_layer = setup_auth_layer(cfg, db.clone());

    // Setup router
    #[rustfmt::skip]
    let mut router = Router::new()
        .route("/dashboard", get(dashboard::home::page))
        .route("/dashboard/employers/add", get(dashboard::employers::add_page).post(dashboard::employers::add))
        .route("/dashboard/employers/update", get(dashboard::employers::update_page).put(dashboard::employers::update))
        .route("/dashboard/employers/{:employer_id}/select", put(dashboard::employers::select))
        .route("/dashboard/jobs/list", get(dashboard::jobs::list_page))
        .route("/dashboard/jobs/add", get(dashboard::jobs::add_page).post(dashboard::jobs::add))
        .route("/dashboard/jobs/preview", post(dashboard::jobs::preview_page))
        .route("/dashboard/jobs/{:job_id}/archive", put(dashboard::jobs::archive))
        .route("/dashboard/jobs/{:job_id}/delete", delete(dashboard::jobs::delete))
        .route("/dashboard/jobs/{:job_id}/publish", put(dashboard::jobs::publish))
        .route("/dashboard/jobs/{:job_id}/update", get(dashboard::jobs::update_page).put(dashboard::jobs::update))
        .route("/locations/search", get(common::search_locations))
        .route_layer(login_required!(AuthnBackend, login_url = LOG_IN_URL, redirect_field = "next_url"))
        .route("/", get(jobboard::jobs::page))
        .route("/about", get(jobboard::about::page))
        .route("/health-check", get(health_check))
        .route("/jobs", get(jobboard::jobs::page))
        .route("/log-in", get(auth::log_in_page).post(auth::log_in))
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

/// Setup router authentication/authorization layer.
#[instrument(skip_all)]
fn setup_auth_layer(
    cfg: &HttpServerConfig,
    db: DynDB,
) -> AuthManagerLayer<AuthnBackend, CachingSessionStore<MokaStore, SessionStore>> {
    // Setup session store
    let session_store = SessionStore::new(db.clone());
    let moka_store = MokaStore::new(Some(1000));
    let caching_session_store = CachingSessionStore::new(moka_store, session_store);

    // Setup session layer
    let secure = if let Some(cookie) = &cfg.cookie {
        cookie.secure.unwrap_or(true)
    } else {
        true
    };
    let session_layer = SessionManagerLayer::new(caching_session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_http_only(true)
        .with_same_site(SameSite::Strict)
        .with_secure(secure);

    // Setup auth layer
    let authn_backend = AuthnBackend::new(db.clone());
    AuthManagerLayerBuilder::new(authn_backend, session_layer).build()
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
