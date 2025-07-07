//! This module defines the router used to dispatch HTTP requests to handlers.

use anyhow::Result;
use axum::{
    Extension, Router,
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
use serde_qs::axum::{QsQueryConfig, QsQueryRejection};
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
        misc::{not_found, search_locations, search_members, search_projects, user_menu_section},
    },
    img::DynImageStore,
    notifications::DynNotificationsManager,
    event_tracker::DynEventTracker,
};

/// Embeds static files from the "static" folder into the binary.
#[derive(Embed)]
#[folder = "dist/static"]
struct StaticFile;

/// Holds shared state for the router, including config and service handles.
#[derive(Clone, FromRef)]
pub(crate) struct State {
    /// HTTP server configuration.
    pub cfg: HttpServerConfig,
    /// Database handle.
    pub db: DynDB,
    /// Image store handle.
    pub image_store: DynImageStore,
    /// `serde_qs` config for query string parsing.
    pub serde_qs_de: serde_qs::Config,
    /// Notifications manager handle.
    pub notifications_manager: DynNotificationsManager,
    /// Event tracker handle.
    pub event_tracker: DynEventTracker,
}

/// Sets up the main application router and all sub-routers.
#[instrument(skip_all, err)]
pub(crate) async fn setup(
    cfg: HttpServerConfig,
    db: DynDB,
    image_store: DynImageStore,
    notifications_manager: DynNotificationsManager,
    event_tracker: DynEventTracker,
) -> Result<Router> {
    // Setup router state
    let state = State {
        cfg: cfg.clone(),
        db: db.clone(),
        image_store,
        serde_qs_de: serde_qs::Config::new(3, false),
        notifications_manager,
        event_tracker,
    };

    // Setup authentication / authorization layer
    let auth_layer = crate::auth::setup_layer(&cfg, db).await?;

    // Setup sub-routers
    let employer_dashboard_router = setup_employer_dashboard_router(&state);
    let job_seeker_dashboard_router = setup_job_seeker_dashboard_router();
    let moderator_dashboard_router = setup_moderator_dashboard_router(&state);
    let dashboard_images_router = setup_dashboard_images_router(&state);
    let jobboard_images_router = setup_jobboard_images_router(&state);

    // Setup main router
    let mut router = Router::new()
        .route(
            "/dashboard/account/update/details",
            put(auth::update_user_details),
        )
        .route(
            "/dashboard/account/update/password",
            put(auth::update_user_password),
        )
        .nest("/dashboard/employer", employer_dashboard_router)
        .nest("/dashboard/images", dashboard_images_router)
        .nest("/dashboard/job-seeker", job_seeker_dashboard_router)
        .nest("/dashboard/moderator", moderator_dashboard_router)
        .route("/dashboard/members/search", get(search_members))
        .route("/jobs/{job_id}/apply", post(jobboard::jobs::apply))
        .route_layer(login_required!(
            AuthnBackend,
            login_url = LOG_IN_URL,
            redirect_field = "next_url"
        ))
        .route("/", get(jobboard::jobs::jobs_page))
        .route("/about", get(jobboard::about::page))
        .route("/embed", get(jobboard::embed::jobs_page))
        .route("/embed/job/{job_id}/card.svg", get(jobboard::embed::job_card))
        .route("/health-check", get(health_check))
        .nest("/jobboard/images", jobboard_images_router)
        .route("/jobs/{job_id}/views", post(jobboard::jobs::track_view))
        .route("/jobs/search-appearances", post(jobboard::jobs::track_search_appearances))
        .route("/locations/search", get(search_locations))
        .route("/log-in", get(auth::log_in_page));

    // Setup some routes based on the login options enabled
    if cfg.login.email {
        router = router
            .route("/log-in", post(auth::log_in))
            .route("/sign-up", post(auth::sign_up))
            .route("/verify-email/{code}", get(auth::verify_email));
    }
    if cfg.login.github {
        router = router
            .route("/log-in/oauth2/{provider}", get(auth::oauth2_redirect))
            .route("/log-in/oauth2/{provider}/callback", get(auth::oauth2_callback));
    }
    if cfg.login.linuxfoundation {
        router = router
            .route("/log-in/oidc/{provider}", get(auth::oidc_redirect))
            .route("/log-in/oidc/{provider}/callback", get(auth::oidc_callback));
    }

    // Resume router setup.
    router = router
        .route("/log-out", get(auth::log_out))
        .route("/projects/search", get(search_projects))
        .route("/section/jobs/{job_id}", get(jobboard::jobs::job_section))
        .route("/section/jobs/results", get(jobboard::jobs::results_section))
        .route("/section/user-menu", get(user_menu_section))
        .route("/sign-up", get(auth::sign_up_page))
        .route("/stats", get(jobboard::stats::page))
        .route_layer(MessagesManagerLayer)
        .route_layer(auth_layer)
        .route_layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .route_layer(Extension(QsQueryConfig::new(3, false).error_handler(|err| {
            QsQueryRejection::new(err, StatusCode::UNPROCESSABLE_ENTITY)
        })))
        .route("/static/{*file}", get(static_handler))
        .fallback(not_found)
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::from_static("max-age=0"),
        ));

    // Setup basic auth
    if let Some(basic_auth) = &cfg.basic_auth {
        if basic_auth.enabled {
            router = router.layer(ValidateRequestHeaderLayer::basic(
                &basic_auth.username,
                &basic_auth.password,
            ));
        }
    }

    Ok(router.with_state(state))
}

/// Sets up the employer dashboard router and its routes.
fn setup_employer_dashboard_router(state: &State) -> Router<State> {
    // Setup middleware
    let check_user_has_profile_access =
        middleware::from_fn_with_state(state.clone(), auth::user_has_profile_access);
    let check_user_owns_employer = middleware::from_fn_with_state(state.clone(), auth::user_owns_employer);
    let check_user_owns_job = middleware::from_fn_with_state(state.clone(), auth::user_owns_job);

    // Setup router
    Router::new()
        .route("/", get(dashboard::employer::home::page))
        .route(
            "/applications/list",
            get(dashboard::employer::applications::list_page),
        )
        .route(
            "/applications/profile/{profile_id}/preview",
            get(dashboard::employer::applications::profile_preview_page)
                .layer(check_user_has_profile_access.clone()),
        )
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
        .route(
            "/invitations",
            get(dashboard::employer::team::user_invitations_list_page),
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
            "/jobs/{job_id}/archive",
            put(dashboard::employer::jobs::archive).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/delete",
            delete(dashboard::employer::jobs::delete).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/preview",
            post(dashboard::employer::jobs::preview_page_wo_job).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/publish",
            put(dashboard::employer::jobs::publish).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/stats",
            get(dashboard::employer::jobs::stats).layer(check_user_owns_job.clone()),
        )
        .route(
            "/jobs/{job_id}/update",
            get(dashboard::employer::jobs::update_page)
                .layer(check_user_owns_job.clone())
                .put(dashboard::employer::jobs::update)
                .layer(check_user_owns_job.clone()),
        )
        .route(
            "/team/invitations/{employer_id}/accept",
            put(dashboard::employer::team::accept_invitation),
        )
        .route(
            "/team/invitations/{employer_id}/reject",
            put(dashboard::employer::team::reject_invitation),
        )
        .route("/team/members/add", post(dashboard::employer::team::add_member))
        .route(
            "/team/members/list",
            get(dashboard::employer::team::members_list_page),
        )
        .route(
            "/team/members/{user_id}/delete",
            delete(dashboard::employer::team::delete_member),
        )
}

/// Sets up the job seeker dashboard router and its routes.
fn setup_job_seeker_dashboard_router() -> Router<State> {
    Router::new()
        .route("/", get(dashboard::job_seeker::home::page))
        .route(
            "/applications/list",
            get(dashboard::job_seeker::applications::list_page),
        )
        .route(
            "/applications/{application_id}/cancel",
            put(dashboard::job_seeker::applications::cancel),
        )
        .route(
            "/profile/preview",
            post(dashboard::job_seeker::profile::preview_page),
        )
        .route(
            "/profile/update",
            get(dashboard::job_seeker::profile::update_page).put(dashboard::job_seeker::profile::update),
        )
}

/// Sets up the moderator dashboard router and its routes.
fn setup_moderator_dashboard_router(state: &State) -> Router<State> {
    // Setup middleware
    let user_is_moderator = middleware::from_fn_with_state(state.clone(), auth::user_is_moderator);

    // Setup router
    Router::new()
        .route("/", get(dashboard::moderator::home::page))
        .route("/jobs/live", get(dashboard::moderator::jobs::live_page))
        .route("/jobs/pending", get(dashboard::moderator::jobs::pending_page))
        .route("/jobs/{job_id}/approve", put(dashboard::moderator::jobs::approve))
        .route("/jobs/{job_id}/reject", put(dashboard::moderator::jobs::reject))
        .route(
            "/jobs/{employer_id}/{job_id}/preview",
            get(dashboard::moderator::jobs::preview_page),
        )
        .route_layer(user_is_moderator)
}

/// Sets up the dashboard images router for authenticated image access.
fn setup_dashboard_images_router(state: &State) -> Router<State> {
    // Setup middleware
    let check_user_has_image_access =
        middleware::from_fn_with_state(state.clone(), auth::user_has_image_access);

    // Setup router
    Router::new().route("/", post(img::upload)).route(
        "/{image_id}/{version}",
        get(img::get).layer(check_user_has_image_access),
    )
}

/// Sets up the job board images router for public image access.
fn setup_jobboard_images_router(state: &State) -> Router<State> {
    // Setup middleware
    let check_image_is_public = middleware::from_fn_with_state(state.clone(), auth::image_is_public);

    // Setup router
    Router::new().route(
        "/{image_id}/{version}",
        get(img::get).layer(check_image_is_public),
    )
}

/// Responds to health check requests with HTTP 200 OK.
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

/// Serves static files embedded in the binary, with cache headers.
async fn static_handler(uri: Uri) -> impl IntoResponse {
    // Extract file path from URI
    let path = uri.path().trim_start_matches("/static/");

    // Set cache duration based on resource type
    #[cfg(not(debug_assertions))]
    let cache_max_age = if path.starts_with("js/") || path.starts_with("css/") {
        // These assets are hashed, so we can cache them for a long time
        60 * 60 * 24 * 365 // 1 year
    } else if path.starts_with("images/") {
        60 * 60 * 24 * 7 // 1 week
    } else if path.starts_with("vendor/") {
        60 * 60 * 24 * 30 // 1 month
    } else {
        // Default cache duration for other static resources
        60 * 60 // 1 hour
    };
    #[cfg(debug_assertions)]
    let cache_max_age = 0;

    // Get file content and return it (if available)
    match StaticFile::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let cache = format!("max-age={cache_max_age}");
            let headers = [(CONTENT_TYPE, mime.as_ref()), (CACHE_CONTROL, &cache)];
            (headers, file.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
