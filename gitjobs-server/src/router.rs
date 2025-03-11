//! This module defines the router used to dispatch HTTP requests to the
//! corresponding handler.

use anyhow::Result;
use axum::{
    Router,
    extract::FromRef,
    http::{
        HeaderValue, StatusCode, Uri,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
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
        dashboard, img, jobboard,
        misc::{search_locations, search_members, search_projects},
    },
    img::DynImageStore,
    notifications::DynNotificationsManager,
};

/// Default cache duration.
#[cfg(debug_assertions)]
pub(crate) const DEFAULT_CACHE_DURATION: usize = 0; // No cache
#[cfg(not(debug_assertions))]
pub(crate) const DEFAULT_CACHE_DURATION: usize = 0; // No cache

/// Embed static files in the binary.
#[derive(Embed)]
#[folder = "static"]
struct StaticFile;

/// Router's state.
#[derive(Clone, FromRef)]
pub(crate) struct State {
    pub cfg: HttpServerConfig,
    pub db: DynDB,
    pub image_store: DynImageStore,
    pub serde_qs_de: serde_qs::Config,
    pub notifications_manager: DynNotificationsManager,
}

/// Setup router.
#[instrument(skip_all)]
pub(crate) async fn setup(
    cfg: HttpServerConfig,
    db: DynDB,
    image_store: DynImageStore,
    notifications_manager: DynNotificationsManager,
) -> Result<Router> {
    // Setup router state
    let state = State {
        cfg: cfg.clone(),
        db: db.clone(),
        image_store,
        serde_qs_de: serde_qs::Config::new(3, false),
        notifications_manager,
    };

    // Setup authentication / authorization layer
    let auth_layer = crate::auth::setup_layer(&cfg, db).await?;

    // Setup sub-routers
    let employer_dashboard_router = setup_employer_dashboard_router(state.clone());
    let job_seeker_dashboard_router = setup_job_seeker_dashboard_router();
    let dashboard_images_router = setup_dashboard_images_router(state.clone());
    let jobboard_images_router = setup_jobboard_images_router(state.clone());

    // Setup main router
    let mut router = Router::new()
        .nest("/dashboard/employer", employer_dashboard_router)
        .nest("/dashboard/job-seeker", job_seeker_dashboard_router)
        .nest("/dashboard/images", dashboard_images_router)
        .route("/account/update/details", put(auth::update_user_details))
        .route("/account/update/password", put(auth::update_user_password))
        .route("/locations/search", get(search_locations))
        .route("/members/search", get(search_members))
        .route("/projects/search", get(search_projects))
        .route_layer(login_required!(
            AuthnBackend,
            login_url = LOG_IN_URL,
            redirect_field = "next_url"
        ))
        .route("/", get(jobboard::home::page))
        .route("/about", get(jobboard::about::page))
        .route("/health-check", get(health_check))
        .nest("/jobboard/images", jobboard_images_router)
        .route("/jobs", get(jobboard::jobs::jobs_page))
        .route("/jobs/{job_id}", get(jobboard::jobs::job_page))
        .route("/jobs/section/explore", get(jobboard::jobs::explore_section))
        .route("/jobs/section/results", get(jobboard::jobs::results_section))
        .route("/log-in", get(auth::log_in_page).post(auth::log_in))
        .route("/log-in/oauth2/{provider}", get(auth::oauth2_redirect))
        .route("/log-in/oauth2/{provider}/callback", get(auth::oauth2_callback))
        .route("/log-in/oidc/{provider}", get(auth::oidc_redirect))
        .route("/log-in/oidc/{provider}/callback", get(auth::oidc_callback))
        .route("/log-out", get(auth::log_out))
        .route("/sign-up", get(auth::sign_up_page).post(auth::sign_up))
        .route("/verify-email/{code}", get(auth::verify_email))
        .route_layer(MessagesManagerLayer)
        .route_layer(auth_layer)
        .route_layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .route("/static/{*file}", get(static_handler))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::try_from(format!("max-age={DEFAULT_CACHE_DURATION}")).expect("valid header value"),
        ))
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

/// Setup employer dashboard router.
#[instrument(skip_all)]
fn setup_employer_dashboard_router(state: State) -> Router<State> {
    // Setup middleware
    let check_user_owns_employer = middleware::from_fn_with_state(state.clone(), auth::user_owns_employer);
    let check_user_owns_job = middleware::from_fn_with_state(state.clone(), auth::user_owns_job);

    // Setup router
    Router::new()
        .route("/", get(dashboard::employer::home::page))
        .route(
            "/employers/add",
            get(dashboard::employer::employers::add_page).post(dashboard::employer::employers::add),
        )
        .route(
            "/employers/update",
            get(dashboard::employer::employers::update_page).put(dashboard::employer::employers::update),
        )
        .route(
            "/employers/{employer_id}/select",
            put(dashboard::employer::employers::select).layer(check_user_owns_employer.clone()),
        )
        .route("/jobs/list", get(dashboard::employer::jobs::list_page))
        .route(
            "/jobs/add",
            get(dashboard::employer::jobs::add_page).post(dashboard::employer::jobs::add),
        )
        .route(
            "/jobs/preview",
            post(dashboard::employer::jobs::preview_page_w_job),
        )
        .route(
            "/jobs/preview/{job_id}",
            post(dashboard::employer::jobs::preview_page_wo_job).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/archive",
            put(dashboard::employer::jobs::archive).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/delete",
            delete(dashboard::employer::jobs::delete).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/publish",
            put(dashboard::employer::jobs::publish).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/update",
            get(dashboard::employer::jobs::update_page)
                .layer(check_user_owns_job.clone())
                .put(dashboard::employer::jobs::update)
                .layer(check_user_owns_job.clone()),
        )
        .route(
            "/applications/list",
            get(dashboard::employer::applications::list_page),
        )
}

/// Setup job seeker dashboard router.
#[instrument(skip_all)]
fn setup_job_seeker_dashboard_router() -> Router<State> {
    Router::new()
        .route("/", get(dashboard::job_seeker::home::page))
        .route(
            "/profile/preview",
            post(dashboard::job_seeker::profile::preview_page),
        )
        .route(
            "/profile/update",
            get(dashboard::job_seeker::profile::update_page).put(dashboard::job_seeker::profile::update),
        )
}

/// Setup dashboard images router.
#[instrument(skip_all)]
fn setup_dashboard_images_router(state: State) -> Router<State> {
    // Setup middleware
    let check_user_has_image_access =
        middleware::from_fn_with_state(state.clone(), auth::user_has_image_access);

    // Setup router
    Router::new().route("/", post(img::upload)).route(
        "/{image_id}/{version}",
        get(img::get).layer(check_user_has_image_access),
    )
}

/// Setup job board images router.
#[instrument(skip_all)]
fn setup_jobboard_images_router(state: State) -> Router<State> {
    // Setup middleware
    let check_image_is_public = middleware::from_fn_with_state(state.clone(), auth::image_is_public);

    // Setup router
    Router::new().route(
        "/{image_id}/{version}",
        get(img::get).layer(check_image_is_public),
    )
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
