//! This module defines some handlers used for authentication.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use axum_messages::Messages;
use password_auth::verify_password;
use rinja::Template;
use serde::Deserialize;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::{self, AuthSession, Credentials, OAuth2Credentials, OAuth2Provider, PasswordCredentials},
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{JobBoardId, OAuth2},
    },
    templates::{self, PageId},
};

/// Log in URL.
pub(crate) const LOG_IN_URL: &str = "/log-in";

/// Log out URL.
pub(crate) const LOG_OUT_URL: &str = "/log-out";

/// Key to store the oauth2 csrf state in the session.
pub(crate) const OAUTH2_CSRF_STATE_KEY: &str = "oauth2.csrf_state";

/// Key to store the oauth2 next url in the session.
pub(crate) const OAUTH2_NEXT_URL_KEY: &str = "oauth2.next_url";

/// Key to store the selected employer id in the session.
pub(crate) const SELECTED_EMPLOYER_ID_KEY: &str = "selected_employer_id";

/// Sign up URL.
pub(crate) const SIGN_UP_URL: &str = "/sign-up";

// Pages handlers.

/// Handler that returns the log in page.
#[instrument(skip_all, err)]
pub(crate) async fn log_in_page(
    messages: Messages,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    let template = templates::auth::LogInPage {
        logged_in: false,
        messages: messages.into_iter().collect(),
        name: None,
        next_url: query.get("next_url").cloned(),
        page_id: PageId::LogIn,
        username: None,
    };

    Ok(Html(template.render()?))
}

/// Handler that returns the sign up page.
#[instrument(skip_all, err)]
pub(crate) async fn sign_up_page(
    messages: Messages,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    let template = templates::auth::SignUpPage {
        logged_in: false,
        messages: messages.into_iter().collect(),
        name: None,
        next_url: query.get("next_url").cloned(),
        page_id: PageId::SignUp,
        username: None,
    };

    Ok(Html(template.render()?))
}

// Actions handlers.

/// Handler that logs the user in.
#[instrument(skip_all)]
pub(crate) async fn log_in(
    mut auth_session: AuthSession,
    messages: Messages,
    session: Session,
    Query(query): Query<HashMap<String, String>>,
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
    Form(mut creds): Form<PasswordCredentials>,
) -> Result<impl IntoResponse, HandlerError> {
    // Authenticate user
    creds.job_board_id = Some(job_board_id);
    let Some(user) = auth_session
        .authenticate(Credentials::Password(creds.clone()))
        .await
        .map_err(|e| HandlerError::Auth(e.to_string()))?
    else {
        messages.error("Invalid credentials");
        let log_in_url = get_log_in_url(query.get("next_url"));
        return Ok(Redirect::to(&log_in_url));
    };

    // Log user in
    auth_session
        .login(&user)
        .await
        .map_err(|e| HandlerError::Auth(e.to_string()))?;

    // Use the first employer as the selected employer in the session
    let employers = db.list_employers(&user.user_id).await?;
    if !employers.is_empty() {
        session
            .insert(SELECTED_EMPLOYER_ID_KEY, employers[0].employer_id)
            .await?;
    }

    // Prepare next url
    let next_url = if let Some(next_url) = query.get("next_url") {
        next_url
    } else {
        "/"
    };

    Ok(Redirect::to(next_url))
}

/// Handler that logs the user out.
#[instrument(skip_all)]
pub(crate) async fn log_out(mut auth_session: AuthSession) -> Result<impl IntoResponse, HandlerError> {
    auth_session
        .logout()
        .await
        .map_err(|e| HandlerError::Auth(e.to_string()))?;

    Ok(Redirect::to(LOG_IN_URL))
}

/// Handler that completes the oauth2 authorization process.
#[instrument(skip_all)]
pub(crate) async fn oauth2_callback(
    mut auth_session: AuthSession,
    messages: Messages,
    session: Session,
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
    Path(provider): Path<OAuth2Provider>,
    Query(OAuth2AuthorizationResponse { code, state }): Query<OAuth2AuthorizationResponse>,
) -> Result<impl IntoResponse, HandlerError> {
    const OAUTH2_AUTHORIZATION_FAILED: &str = "OAuth2 authorization failed";

    // Verify csrf state
    let Some(state_in_session) = session.remove::<oauth2::CsrfToken>(OAUTH2_CSRF_STATE_KEY).await? else {
        messages.error(OAUTH2_AUTHORIZATION_FAILED);
        return Ok(Redirect::to(LOG_IN_URL));
    };
    if state_in_session.secret() != state.secret() {
        messages.error(OAUTH2_AUTHORIZATION_FAILED);
        return Ok(Redirect::to(LOG_IN_URL));
    }

    // Get next url from session (if any)
    let next_url = session.remove::<Option<String>>(OAUTH2_NEXT_URL_KEY).await?.flatten();

    // Authenticate user
    let creds = OAuth2Credentials {
        code,
        job_board_id,
        provider,
    };
    let Some(user) = auth_session
        .authenticate(Credentials::OAuth2(creds))
        .await
        .map_err(|e| HandlerError::Auth(e.to_string()))?
    else {
        messages.error(OAUTH2_AUTHORIZATION_FAILED);
        let log_in_url = get_log_in_url(next_url.as_ref());
        return Ok(Redirect::to(&log_in_url));
    };

    // Log user in
    auth_session
        .login(&user)
        .await
        .map_err(|e| HandlerError::Auth(e.to_string()))?;

    // Use the first employer as the selected employer in the session
    let employers = db.list_employers(&user.user_id).await?;
    if !employers.is_empty() {
        session
            .insert(SELECTED_EMPLOYER_ID_KEY, employers[0].employer_id)
            .await?;
    }

    // Prepare next url
    let next_url = next_url.unwrap_or("/".to_string());

    Ok(Redirect::to(&next_url))
}

/// Handler that redirects the user to the oauth2 provider.
#[instrument(skip_all)]
pub(crate) async fn oauth2_redirect(
    session: Session,
    OAuth2(oauth2_provider): OAuth2,
    Form(NextUrl { next_url }): Form<NextUrl>,
) -> Result<impl IntoResponse, HandlerError> {
    // Generate the authorization url
    let mut builder = oauth2_provider.client.authorize_url(oauth2::CsrfToken::new_random);
    for scope in &oauth2_provider.scopes {
        builder = builder.add_scope(oauth2::Scope::new(scope.clone()));
    }
    let (authorize_url, csrf_state) = builder.url();

    // Save the csrf state and next url in the session
    session.insert(OAUTH2_CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(OAUTH2_NEXT_URL_KEY, next_url).await?;

    // Redirect to the authorization url
    Ok(Redirect::to(authorize_url.as_str()))
}

/// Handler that signs up a new user.
#[instrument(skip_all)]
pub(crate) async fn sign_up(
    messages: Messages,
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
    Query(query): Query<HashMap<String, String>>,
    Form(mut user_summary): Form<auth::UserSummary>,
) -> Result<impl IntoResponse, HandlerError> {
    // Check if the password has been provided
    let Some(password) = user_summary.password.take() else {
        return Ok((StatusCode::BAD_REQUEST, "password not provided").into_response());
    };

    // Sign up the user
    user_summary.password = Some(password_auth::generate_hash(&password));
    if db.sign_up_user(&job_board_id, &user_summary, true).await.is_err() {
        // Redirect to the sign up page on error
        messages.error("Something went wrong while signing up. Please try again later.");
        return Ok(Redirect::to(SIGN_UP_URL).into_response());
    }

    // Redirect to the log in page on success
    let log_in_url = get_log_in_url(query.get("next_url"));
    Ok(Redirect::to(&log_in_url).into_response())
}

/// Handler that updates the user's details.
#[instrument(skip_all, err)]
pub(crate) async fn update_user_details(
    auth_session: AuthSession,
    messages: Messages,
    State(db): State<DynDB>,
    Form(user_summary): Form<auth::UserSummary>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Update user in database
    let user_id = user.user_id;
    db.update_user_details(&user_id, &user_summary).await?;
    messages.success("User details updated successfully.");

    Ok((StatusCode::NO_CONTENT, [("HX-Trigger", "refresh-body")]).into_response())
}

/// Handler that updates the user's password.
#[instrument(skip_all, err)]
pub(crate) async fn update_user_password(
    auth_session: AuthSession,
    State(db): State<DynDB>,
    Form(mut input): Form<auth::PasswordUpdateInput>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok(StatusCode::FORBIDDEN.into_response());
    };

    // Check if the old password provided is correct
    let Some(old_password_hash) = db.get_user_password(&user.user_id).await? else {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    };
    if tokio::task::spawn_blocking(move || verify_password(&input.old_password, &old_password_hash))
        .await
        .map_err(anyhow::Error::from)?
        .is_err()
    {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }

    // Update password in database
    input.new_password = password_auth::generate_hash(&input.new_password);
    db.update_user_password(&user.user_id, &input.new_password).await?;

    Ok(Redirect::to(LOG_OUT_URL).into_response())
}

// Authorization middleware.

/// Check if the user has access to the image provided.
#[instrument(skip_all)]
pub(crate) async fn user_has_image_access(
    State(db): State<DynDB>,
    Path((image_id, _)): Path<(Uuid, String)>,
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    // Check if user is logged in
    let Some(user) = auth_session.user else {
        return StatusCode::FORBIDDEN.into_response();
    };

    // Check if the user has access to the image
    let Ok(has_access) = db.user_has_image_access(&user.user_id, &image_id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if !has_access {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await.into_response()
}

/// Check if the user owns the employer provided.
#[instrument(skip_all)]
pub(crate) async fn user_owns_employer(
    State(db): State<DynDB>,
    Path(employer_id): Path<Uuid>,
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    // Check if user is logged in
    let Some(user) = auth_session.user else {
        return StatusCode::FORBIDDEN.into_response();
    };

    // Check if the user owns the employer
    let Ok(user_owns_employer) = db.user_owns_employer(&user.user_id, &employer_id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if !user_owns_employer {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await.into_response()
}

/// Check if the user owns the job provided.
#[instrument(skip_all)]
pub(crate) async fn user_owns_job(
    State(db): State<DynDB>,
    Path(job_id): Path<Uuid>,
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    // Check if user is logged in
    let Some(user) = auth_session.user else {
        return StatusCode::FORBIDDEN.into_response();
    };

    // Check if the user owns the job
    let Ok(user_owns_job) = db.user_owns_job(&user.user_id, &job_id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if !user_owns_job {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await.into_response()
}

/// Get the log in url including the next url if provided.
fn get_log_in_url(next_url: Option<&String>) -> String {
    let mut log_in_url = LOG_IN_URL.to_string();
    if let Some(next_url) = next_url {
        log_in_url = format!("{log_in_url}?next_url={next_url}");
    };
    log_in_url
}

// Deserialization helpers.

/// `OAuth2` authorization response.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2AuthorizationResponse {
    code: String,
    state: oauth2::CsrfToken,
}

/// Next url to redirect to.
#[derive(Debug, Deserialize)]
pub(crate) struct NextUrl {
    pub next_url: Option<String>,
}
