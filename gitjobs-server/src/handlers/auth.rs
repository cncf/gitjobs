//! This module defines some handlers used for authentication.

use std::collections::HashMap;

use askama::Template;
use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use axum_messages::Messages;
use openidconnect as oidc;
use password_auth::verify_password;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::{self, AuthSession, Credentials, OAuth2Credentials, OidcCredentials, PasswordCredentials},
    config::{HttpServerConfig, OAuth2Provider, OidcProvider},
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{OAuth2, Oidc},
    },
    notifications::{DynNotificationsManager, NewNotification, NotificationKind},
    templates::{self, PageId, auth::User, notifications::EmailVerification},
};

/// Key used to store the authentication provider in the session.
pub(crate) const AUTH_PROVIDER_KEY: &str = "auth_provider";

/// URL for the log in page.
pub(crate) const LOG_IN_URL: &str = "/log-in";

/// URL for the log out page.
pub(crate) const LOG_OUT_URL: &str = "/log-out";

/// Key used to store the next URL in the session.
pub(crate) const NEXT_URL_KEY: &str = "next_url";

/// Key used to store the `OAuth2` CSRF state in the session.
pub(crate) const OAUTH2_CSRF_STATE_KEY: &str = "oauth2.csrf_state";

/// Key used to store the `Oidc` nonce in the session.
pub(crate) const OIDC_NONCE_KEY: &str = "oidc.nonce";

/// Key used to store the selected employer id in the session.
pub(crate) const SELECTED_EMPLOYER_ID_KEY: &str = "selected_employer_id";

/// URL for the sign up page.
pub(crate) const SIGN_UP_URL: &str = "/sign-up";

// Pages handlers.

/// Handler that returns the log in page.
#[instrument(skip_all, err)]
pub(crate) async fn log_in_page(
    auth_session: AuthSession,
    messages: Messages,
    State(cfg): State<HttpServerConfig>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Check if the user is already logged in
    if auth_session.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    // Sanitize and encode the next url (if provided)
    let next_url =
        sanitize_next_url(query.get("next_url").map(String::as_str)).map(|value| encode_next_url(&value));

    // Prepare template
    let template = templates::auth::LogInPage {
        auth_provider: None,
        login: cfg.login.clone(),
        cfg: cfg.into(),
        messages: messages.into_iter().collect(),
        next_url,
        page_id: PageId::LogIn,
        user: User::default(),
    };

    Ok(Html(template.render()?).into_response())
}

/// Handler that returns the sign up page.
#[instrument(skip_all, err)]
pub(crate) async fn sign_up_page(
    auth_session: AuthSession,
    messages: Messages,
    State(cfg): State<HttpServerConfig>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HandlerError> {
    // Check if the user is already logged in
    if auth_session.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    // Sanitize and encode the next url (if provided)
    let next_url =
        sanitize_next_url(query.get("next_url").map(String::as_str)).map(|value| encode_next_url(&value));

    // Prepare template
    let template = templates::auth::SignUpPage {
        auth_provider: None,
        login: cfg.login.clone(),
        cfg: cfg.into(),
        messages: messages.into_iter().collect(),
        next_url,
        page_id: PageId::SignUp,
        user: User::default(),
    };

    Ok(Html(template.render()?).into_response())
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
    Form(creds): Form<PasswordCredentials>,
) -> Result<impl IntoResponse, HandlerError> {
    // Sanitize next url
    let next_url = sanitize_next_url(query.get("next_url").map(String::as_str));

    // Authenticate user
    let Some(user) = auth_session
        .authenticate(Credentials::Password(creds.clone()))
        .await
        .map_err(|e| HandlerError::Auth(e.to_string()))?
    else {
        messages.error("Invalid credentials. Please make sure you have verified your email address.");
        let log_in_url = get_log_in_url(next_url.as_deref());
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

    let next_url = next_url.as_deref().unwrap_or("/");
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
    Path(provider): Path<OAuth2Provider>,
    Query(OAuth2AuthorizationResponse { code, state }): Query<OAuth2AuthorizationResponse>,
) -> Result<impl IntoResponse, HandlerError> {
    const OAUTH2_AUTHORIZATION_FAILED: &str = "OAuth2 authorization failed";

    // Verify oauth2 csrf state
    let Some(state_in_session) = session.remove::<oauth2::CsrfToken>(OAUTH2_CSRF_STATE_KEY).await? else {
        messages.error(OAUTH2_AUTHORIZATION_FAILED);
        return Ok(Redirect::to(LOG_IN_URL));
    };
    if state_in_session.secret() != state.secret() {
        messages.error(OAUTH2_AUTHORIZATION_FAILED);
        return Ok(Redirect::to(LOG_IN_URL));
    }

    // Get next url from session (if any)
    let next_url = session
        .remove::<Option<String>>(NEXT_URL_KEY)
        .await?
        .flatten()
        .and_then(|value| sanitize_next_url(Some(value.as_str())));
    let log_in_url = get_log_in_url(next_url.as_deref());

    // Authenticate user
    let creds = OAuth2Credentials { code, provider };
    let user = match auth_session.authenticate(Credentials::OAuth2(creds)).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            messages.error(OAUTH2_AUTHORIZATION_FAILED);
            return Ok(Redirect::to(&log_in_url));
        }
        Err(err) => {
            messages.error(format!("{OAUTH2_AUTHORIZATION_FAILED}: {err}"));
            return Ok(Redirect::to(&log_in_url));
        }
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

    let next_url = next_url.as_deref().unwrap_or("/");
    Ok(Redirect::to(next_url))
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

    // Sanitize the next url (if any)
    let next_url = sanitize_next_url(next_url.as_deref());

    // Save the csrf state and next url in the session
    session.insert(OAUTH2_CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(NEXT_URL_KEY, next_url).await?;

    // Redirect to the authorization url
    Ok(Redirect::to(authorize_url.as_str()))
}

/// Handler that completes the oidc authorization process.
#[instrument(skip_all)]
pub(crate) async fn oidc_callback(
    mut auth_session: AuthSession,
    messages: Messages,
    session: Session,
    State(db): State<DynDB>,
    Path(provider): Path<OidcProvider>,
    Query(OAuth2AuthorizationResponse { code, state }): Query<OAuth2AuthorizationResponse>,
) -> Result<impl IntoResponse, HandlerError> {
    const OIDC_AUTHORIZATION_FAILED: &str = "OpenID Connect authorization failed";

    // Verify oauth2 csrf state
    let Some(state_in_session) = session.remove::<oauth2::CsrfToken>(OAUTH2_CSRF_STATE_KEY).await? else {
        messages.error(OIDC_AUTHORIZATION_FAILED);
        return Ok(Redirect::to(LOG_IN_URL));
    };
    if state_in_session.secret() != state.secret() {
        messages.error(OIDC_AUTHORIZATION_FAILED);
        return Ok(Redirect::to(LOG_IN_URL));
    }

    // Get oidc nonce from session
    let Some(nonce) = session.remove::<oidc::Nonce>(OIDC_NONCE_KEY).await? else {
        messages.error(OIDC_AUTHORIZATION_FAILED);
        return Ok(Redirect::to(LOG_IN_URL));
    };

    // Get next url from session (if any)
    let next_url = session
        .remove::<Option<String>>(NEXT_URL_KEY)
        .await?
        .flatten()
        .and_then(|value| sanitize_next_url(Some(value.as_str())));
    let log_in_url = get_log_in_url(next_url.as_deref());

    // Authenticate user
    let creds = OidcCredentials {
        code,
        nonce,
        provider: provider.clone(),
    };
    let user = match auth_session.authenticate(Credentials::Oidc(creds)).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            messages.error(OIDC_AUTHORIZATION_FAILED);
            return Ok(Redirect::to(&log_in_url));
        }
        Err(err) => {
            messages.error(format!("{OIDC_AUTHORIZATION_FAILED}: {err}"));
            return Ok(Redirect::to(&log_in_url));
        }
    };

    // Log user in
    auth_session
        .login(&user)
        .await
        .map_err(|e| HandlerError::Auth(e.to_string()))?;

    // Track auth provider in the session
    session.insert(AUTH_PROVIDER_KEY, provider).await?;

    // Use the first employer as the selected employer in the session
    let employers = db.list_employers(&user.user_id).await?;
    if !employers.is_empty() {
        session
            .insert(SELECTED_EMPLOYER_ID_KEY, employers[0].employer_id)
            .await?;
    }

    let next_url = next_url.as_deref().unwrap_or("/");
    Ok(Redirect::to(next_url))
}

/// Handler that redirects the user to the oidc provider.
#[instrument(skip_all)]
pub(crate) async fn oidc_redirect(
    session: Session,
    Oidc(oidc_provider): Oidc,
    Form(NextUrl { next_url }): Form<NextUrl>,
) -> Result<impl IntoResponse, HandlerError> {
    // Generate the authorization url
    let mut builder = oidc_provider.client.authorize_url(
        oidc::AuthenticationFlow::<oidc::core::CoreResponseType>::AuthorizationCode,
        oidc::CsrfToken::new_random,
        oidc::Nonce::new_random,
    );
    for scope in &oidc_provider.scopes {
        builder = builder.add_scope(oidc::Scope::new(scope.clone()));
    }
    let (authorize_url, csrf_state, nonce) = builder.url();

    // Sanitize the next url (if any)
    let next_url = sanitize_next_url(next_url.as_deref());

    // Save the csrf state, nonce and next url in the session
    session.insert(OAUTH2_CSRF_STATE_KEY, csrf_state.secret()).await?;
    session.insert(OIDC_NONCE_KEY, nonce.secret()).await?;
    session.insert(NEXT_URL_KEY, next_url).await?;

    // Redirect to the authorization url
    Ok(Redirect::to(authorize_url.as_str()))
}

/// Handler that signs up a new user.
#[instrument(skip_all)]
pub(crate) async fn sign_up(
    messages: Messages,
    State(cfg): State<HttpServerConfig>,
    State(db): State<DynDB>,
    State(notifications_manager): State<DynNotificationsManager>,
    Query(query): Query<HashMap<String, String>>,
    Form(mut user_summary): Form<auth::UserSummary>,
) -> Result<impl IntoResponse, HandlerError> {
    // Check if the password has been provided
    let Some(password) = user_summary.password.take() else {
        return Ok((StatusCode::BAD_REQUEST, "password not provided").into_response());
    };

    // Generate password hash
    user_summary.password = Some(password_auth::generate_hash(&password));

    // Sign up the user
    let Ok((user, email_verification_code)) = db.sign_up_user(&user_summary, false).await else {
        // Redirect to the sign up page on error
        messages.error("Something went wrong while signing up. Please try again later.");
        return Ok(Redirect::to(SIGN_UP_URL).into_response());
    };

    // Enqueue email verification notification
    if let Some(code) = email_verification_code {
        let template_data = EmailVerification {
            link: format!(
                "{}/verify-email/{code}",
                cfg.base_url.strip_suffix('/').unwrap_or(&cfg.base_url)
            ),
        };
        let notification = NewNotification {
            kind: NotificationKind::EmailVerification,
            recipients: vec![user.user_id],

            template_data: Some(serde_json::to_value(&template_data)?),
        };
        notifications_manager.enqueue(&notification).await?;
        messages.success("Please verify your email to complete the sign up process.");
    }

    // Redirect to the log in page on success
    let next_url = sanitize_next_url(query.get("next_url").map(String::as_str));
    let log_in_url = get_log_in_url(next_url.as_deref());
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

/// Handler that verifies the user's email.
#[instrument(skip_all, err)]
pub(crate) async fn verify_email(
    messages: Messages,
    State(db): State<DynDB>,
    Path(code): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Verify the email
    if db.verify_email(&code).await.is_ok() {
        messages.success("Email verified successfully. You can now log in using your credentials.");
    } else {
        messages.error("Error verifying email (please note that links are only valid for 24 hours).");
    }

    Ok(Redirect::to(LOG_IN_URL).into_response())
}

// Deserialization helpers.

/// `OAuth2` authorization response containing code and CSRF state.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2AuthorizationResponse {
    /// Authorization code returned by the `OAuth2` provider.
    code: String,
    /// CSRF state returned by the `OAuth2` provider.
    state: oauth2::CsrfToken,
}

/// Next URL to redirect to after authentication.
#[derive(Debug, Deserialize)]
pub(crate) struct NextUrl {
    /// The next URL to redirect to, if provided.
    pub next_url: Option<String>,
}

// Authorization middleware.

/// Check if the image provided is public.
#[instrument(skip_all)]
pub(crate) async fn image_is_public(
    State(db): State<DynDB>,
    Path((image_id, _)): Path<(Uuid, String)>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let Ok(is_public) = db.is_image_public(&image_id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if !is_public {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await.into_response()
}

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

/// Check if the user has access to the profile provided.
#[instrument(skip_all)]
pub(crate) async fn user_has_profile_access(
    State(db): State<DynDB>,
    Path(profile_id): Path<Uuid>,
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    // Check if user is logged in
    let Some(user) = auth_session.user else {
        return StatusCode::FORBIDDEN.into_response();
    };

    // Check if the user has access to the profile
    let Ok(has_access) = db.user_has_profile_access(&user.user_id, &profile_id).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if !has_access {
        return StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await.into_response()
}

/// Check if the user is a moderator.
#[instrument(skip_all)]
pub(crate) async fn user_is_moderator(
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    // Check if user is logged in
    let Some(user) = auth_session.user else {
        return StatusCode::FORBIDDEN.into_response();
    };

    // Check if the user is a moderator
    if !user.moderator {
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

// Helpers.

/// Percent-encode a `next_url` so it can be safely embedded in a query string.
fn encode_next_url(next_url: &str) -> String {
    utf8_percent_encode(next_url, NON_ALPHANUMERIC).to_string()
}

/// Get the log in url including the next url if provided.
fn get_log_in_url(next_url: Option<&str>) -> String {
    let mut log_in_url = LOG_IN_URL.to_string();
    if let Some(next_url) = sanitize_next_url(next_url) {
        log_in_url = format!("{log_in_url}?next_url={}", encode_next_url(&next_url));
    }
    log_in_url
}

/// Sanitize a `next_url` value ensuring it points to an in-site path.
fn sanitize_next_url(next_url: Option<&str>) -> Option<String> {
    let value = next_url?.trim();
    if value.is_empty() {
        return None;
    }
    if !value.starts_with('/') || value.starts_with("//") {
        return None;
    }
    Some(value.to_string())
}

// Tests.

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::COOKIE},
    };
    use axum_login::tower_sessions::session;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::mock::MockDB,
        handlers::tests::{
            TestRouterBuilder, notification_matches_kind, sample_auth_user, sample_employer_summary,
            sample_session_record, test_http_server_cfg, with_auth_provider,
        },
        notifications::{MockNotificationsManager, NotificationKind},
    };

    use super::*;

    #[tokio::test]
    async fn test_log_in_page_returns_html() {
        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_log_in_page_redirects_when_user_is_authenticated() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/");
    }

    #[tokio::test]
    async fn test_log_in_redirects_when_credentials_are_invalid() {
        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_user_by_username()
            .times(1)
            .withf(|username| username == "user")
            .returning(|_| Ok(None));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/log-in")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("username=user&password=wrong"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/log-in");
    }

    #[tokio::test]
    async fn test_log_in_logs_user_in_and_redirects_home() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let password_hash = password_auth::generate_hash("secret");
        let user = crate::auth::User {
            auth_hash: b"hash".to_vec(),
            email: "user@example.test".to_string(),
            email_verified: true,
            has_profile: true,
            moderator: false,
            name: "Test User".to_string(),
            user_id,
            username: "user".to_string(),

            has_password: Some(true),
            password: Some(password_hash),
        };

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_user_by_username()
            .times(1)
            .withf(|username| username == "user")
            .returning(move |_| Ok(Some(user.clone())));
        db.expect_list_employers()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(vec![sample_employer_summary(employer_id)]));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/log-in")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("username=user&password=secret"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/");
    }

    #[tokio::test]
    async fn test_log_out_redirects_to_log_in() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-out")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_IN_URL);
    }

    #[tokio::test]
    async fn test_sign_up_page_returns_html() {
        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/sign-up")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_sign_up_returns_bad_request_when_password_is_missing() {
        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/sign-up")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("email=user%40example.test&name=User&username=user"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_sign_up_enqueues_email_verification_notification() {
        // Setup identifiers and data structures
        let verification_code = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let new_user = crate::auth::User {
            auth_hash: b"hash".to_vec(),
            email: "user@example.test".to_string(),
            email_verified: false,
            has_profile: false,
            moderator: false,
            name: "Test User".to_string(),
            user_id,
            username: "user".to_string(),

            has_password: Some(true),
            password: None,
        };

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_sign_up_user()
            .times(1)
            .returning(move |_, _| Ok((new_user.clone(), Some(verification_code))));

        // Setup notifications manager mock
        let mut notifications_manager = MockNotificationsManager::new();
        notifications_manager
            .expect_enqueue()
            .times(1)
            .withf(move |notification| {
                notification.recipients == vec![user_id]
                    && notification_matches_kind(notification, &NotificationKind::EmailVerification)
            })
            .returning(|_| Box::pin(async { Ok(()) }));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, notifications_manager).build().await;
        let request = Request::builder()
            .method("POST")
            .uri("/sign-up")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(
                "email=user%40example.test&name=User&username=user&password=secret",
            ))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/log-in");
    }

    #[tokio::test]
    async fn test_sign_up_redirects_to_sign_up_when_db_fails() {
        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_sign_up_user()
            .times(1)
            .returning(|_, _| Err(anyhow!("db error")));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/sign-up")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(
                "email=user%40example.test&name=User&username=user&password=secret",
            ))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], SIGN_UP_URL);
    }

    #[tokio::test]
    async fn test_update_user_details_returns_no_content() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_update_user_details()
            .times(1)
            .withf(move |id, _| *id == user_id)
            .returning(|_, _| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri("/dashboard/account/update/details")
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("email=user%40example.test&name=User&username=user"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(response.headers()["hx-trigger"], "refresh-body");
    }

    #[tokio::test]
    async fn test_update_user_password_returns_forbidden_with_wrong_old_password() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_get_user_password()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(|_| Ok(Some(password_auth::generate_hash("correct-old"))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri("/dashboard/account/update/password")
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("old_password=wrong&new_password=new-one"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_update_user_password_redirects_to_log_out_on_success() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_get_user_password()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(|_| Ok(Some(password_auth::generate_hash("old-password"))));
        db.expect_update_user_password()
            .times(1)
            .withf(move |id, _| *id == user_id)
            .returning(|_, _| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri("/dashboard/account/update/password")
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("old_password=old-password&new_password=new-one"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_OUT_URL);
    }

    #[tokio::test]
    async fn test_update_user_password_returns_bad_request_when_password_is_missing() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_get_user_password()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(|_| Ok(None));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri("/dashboard/account/update/password")
            .header(COOKIE, format!("id={session_id}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("old_password=old-password&new_password=new-one"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_user_menu_section_includes_auth_provider_from_session() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let mut session_record = sample_session_record(session_id, user_id, auth_hash, None);
        with_auth_provider(&mut session_record, "github");

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/section/user-menu")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_verify_email_redirects_to_log_in_when_verification_succeeds() {
        // Setup identifiers and data structures
        let code = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_verify_email()
            .times(1)
            .withf(move |value| *value == code)
            .returning(|_| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/verify-email/{code}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_IN_URL);
    }

    #[tokio::test]
    async fn test_verify_email_redirects_to_log_in_when_verification_fails() {
        // Setup identifiers and data structures
        let code = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_verify_email()
            .times(1)
            .withf(move |value| *value == code)
            .returning(|_| Err(anyhow!("verification error")));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/verify-email/{code}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_IN_URL);
    }

    #[tokio::test]
    async fn test_oauth2_callback_redirects_when_csrf_state_is_missing() {
        // Setup database mock
        let mut cfg = test_http_server_cfg();
        cfg.login.github = true;

        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in/oauth2/github/callback?code=abc&state=expected-state")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_IN_URL);
    }

    #[tokio::test]
    async fn test_oauth2_callback_redirects_when_csrf_state_mismatches() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let mut session_record = sample_session_record(session_id, user_id, auth_hash, None);
        session_record.data.insert(
            OAUTH2_CSRF_STATE_KEY.to_string(),
            serde_json::json!("expected-state"),
        );

        // Setup database mock
        let mut cfg = test_http_server_cfg();
        cfg.login.github = true;

        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in/oauth2/github/callback?code=abc&state=unexpected-state")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_IN_URL);
    }

    #[tokio::test]
    async fn test_oidc_callback_redirects_when_csrf_state_mismatches() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let mut session_record = sample_session_record(session_id, user_id, auth_hash, None);
        session_record.data.insert(
            OAUTH2_CSRF_STATE_KEY.to_string(),
            serde_json::json!("expected-state"),
        );
        session_record
            .data
            .insert(OIDC_NONCE_KEY.to_string(), serde_json::json!("expected-nonce"));

        // Setup database mock
        let mut cfg = test_http_server_cfg();
        cfg.login.linuxfoundation = true;

        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in/oidc/linuxfoundation/callback?code=abc&state=unexpected-state")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_IN_URL);
    }

    #[tokio::test]
    async fn test_oidc_callback_redirects_when_nonce_is_missing() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let mut session_record = sample_session_record(session_id, user_id, auth_hash, None);
        session_record.data.insert(
            OAUTH2_CSRF_STATE_KEY.to_string(),
            serde_json::json!("expected-state"),
        );

        // Setup database mock
        let mut cfg = test_http_server_cfg();
        cfg.login.linuxfoundation = true;

        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in/oidc/linuxfoundation/callback?code=abc&state=expected-state")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], LOG_IN_URL);
    }

    #[tokio::test]
    async fn test_employer_dashboard_redirects_to_log_in_when_user_is_unauthenticated() {
        // Setup router and send request
        let db = MockDB::new();
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/dashboard/employer")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response.headers()["location"],
            "/log-in?next_url=%2Fdashboard%2Femployer"
        );
    }

    #[tokio::test]
    async fn test_jobboard_image_route_returns_internal_server_error_when_public_check_fails() {
        // Setup identifiers and data structures
        let image_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_is_image_public()
            .times(1)
            .withf(move |id| *id == image_id)
            .returning(|_| Err(anyhow!("db error")));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/jobboard/images/{image_id}/small"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_dashboard_image_route_returns_internal_server_error_when_access_check_fails() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let image_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_user_has_image_access()
            .times(1)
            .withf(move |user, image| *user == user_id && *image == image_id)
            .returning(|_, _| Err(anyhow!("db error")));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!("/dashboard/images/{image_id}/small"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_profile_preview_route_returns_internal_server_error_when_access_check_fails() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let profile_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_user_has_profile_access()
            .times(1)
            .withf(move |id, profile| *id == user_id && *profile == profile_id)
            .returning(|_, _| Err(anyhow!("db error")));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!(
                "/dashboard/employer/applications/profile/{profile_id}/preview"
            ))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_select_employer_route_returns_forbidden_when_user_does_not_own_employer() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let employer_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_user_owns_employer()
            .times(1)
            .withf(move |id, employer| *id == user_id && *employer == employer_id)
            .returning(|_, _| Ok(false));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/employer/employers/{employer_id}/select"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_select_employer_route_returns_internal_server_error_when_ownership_check_fails() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let employer_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_user_owns_employer()
            .times(1)
            .withf(move |id, employer| *id == user_id && *employer == employer_id)
            .returning(|_, _| Err(anyhow!("db error")));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/employer/employers/{employer_id}/select"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_archive_route_returns_internal_server_error_when_ownership_check_fails() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let job_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

        // Setup database mock
        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_user_owns_job()
            .times(1)
            .withf(move |id, job| *id == user_id && *job == job_id)
            .returning(|_, _| Err(anyhow!("db error")));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/employer/jobs/{job_id}/archive"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_get_log_in_url_encodes_next_url() {
        let url = get_log_in_url(Some("/dashboard/employer?tab=jobs"));

        assert_eq!(url, "/log-in?next_url=%2Fdashboard%2Femployer%3Ftab%3Djobs");
    }

    #[test]
    fn test_sanitize_next_url_rejects_external_urls() {
        assert_eq!(sanitize_next_url(Some("https://example.test")), None);
        assert_eq!(sanitize_next_url(Some("//example.test")), None);
    }

    #[test]
    fn test_sanitize_next_url_trims_and_accepts_internal_path() {
        assert_eq!(
            sanitize_next_url(Some("  /dashboard/job-seeker  ")),
            Some("/dashboard/job-seeker".to_string()),
        );
    }

    // Helpers

    fn allow_session_store_updates(db: &mut MockDB) {
        db.expect_create_session().times(0..).returning(|_| Ok(()));
        db.expect_delete_session().times(0..).returning(|_| Ok(()));
        db.expect_update_session().times(0..).returning(|_| Ok(()));
    }
}
