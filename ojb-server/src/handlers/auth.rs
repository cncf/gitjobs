//! This module defines some handlers used for authentication.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use axum_messages::Messages;
use rinja::Template;
use serde::Deserialize;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    auth::{self, AuthSession, Credentials, OAuth2Credentials, OAuth2Provider, PasswordCredentials},
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{JobBoardId, OAuth2},
    },
    templates,
};

/// Log in URL.
pub(crate) const LOG_IN_URL: &str = "/log-in";

/// Key to store the oauth2 csrf state in the session.
pub(crate) const OAUTH2_CSRF_STATE_KEY: &str = "oauth2.csrf_state";

/// Key to store the oauth2 next url in the session.
pub(crate) const OAUTH2_NEXT_URL_KEY: &str = "oauth2.next_url";

/// Key to store the selected employer id in the session.
pub(crate) const SELECTED_EMPLOYER_ID_KEY: &str = "selected_employer_id";

// Pages handlers.

/// Handler that returns the log in page.
#[instrument(skip_all, err)]
pub(crate) async fn log_in_page(
    messages: Messages,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    let template = templates::auth::LogInPage {
        messages: messages.into_iter().collect(),
        next_url: query.get("next_url").cloned(),
    };

    Ok(Html(template.render()?))
}

/// Handler that returns the sign up page.
#[instrument(skip_all, err)]
pub(crate) async fn sign_up_page() -> Result<impl IntoResponse, HandlerError> {
    let template = templates::auth::SignUpPage {};

    Ok(Html(template.render()?))
}

// Actions handlers.

/// Handler that logs the user in.
#[instrument(skip_all)]
pub(crate) async fn log_in(
    mut auth_session: AuthSession,
    messages: Messages,
    session: Session,
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

        let mut log_in_url = LOG_IN_URL.to_string();
        if let Some(next_url) = creds.next_url {
            log_in_url = format!("{log_in_url}?next_url={next_url}");
        };

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
    let next_url = if let Some(ref next_url) = creds.next_url {
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

        let mut log_in_url = LOG_IN_URL.to_string();
        if let Some(next_url) = next_url {
            log_in_url = format!("{log_in_url}?next_url={next_url}");
        };

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
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
    Form(mut new_user): Form<auth::NewUser>,
) -> Result<impl IntoResponse, HandlerError> {
    // Check if the password has been provided
    let Some(password) = new_user.password.take() else {
        return Ok((StatusCode::BAD_REQUEST, "password not provided").into_response());
    };

    // Sign up the user
    new_user.password = Some(password_auth::generate_hash(&password));
    _ = db.sign_up_user(&job_board_id, &new_user, true).await?;

    Ok(Redirect::to(LOG_IN_URL).into_response())
}

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
