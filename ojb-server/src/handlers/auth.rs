//! This module defines some handlers used for authentication.

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use axum_messages::Messages;
use rinja::Template;
use tower_sessions::Session;
use tracing::instrument;

use crate::{
    auth::{self, AuthSession, Credentials},
    db::DynDB,
    handlers::{
        error::HandlerError,
        extractors::{JobBoardId, SELECTED_EMPLOYER_ID_KEY},
    },
    templates,
};

/// Log in URL.
pub(crate) const LOG_IN_URL: &str = "/log-in";

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
    Form(mut creds): Form<Credentials>,
) -> Result<impl IntoResponse, HandlerError> {
    // Verify credentials
    creds.job_board_id = Some(job_board_id);
    let Some(user) = auth_session.authenticate(creds.clone()).await? else {
        messages.error("Invalid credentials");

        let mut log_in_url = LOG_IN_URL.to_string();
        if let Some(next_url) = creds.next_url {
            log_in_url = format!("{log_in_url}?next_url={next_url}");
        };

        return Ok(Redirect::to(&log_in_url));
    };

    // Log user in
    auth_session.login(&user).await?;

    // Use the first employer as the selected employer in the session
    let employers = db.get_user_employers(&user.user_id).await?;
    if !employers.is_empty() {
        session
            .insert(SELECTED_EMPLOYER_ID_KEY, employers[0].employer_id)
            .await?;
    }

    // Prepare redirect url
    let redirect_url = if let Some(ref next_url) = creds.next_url {
        next_url
    } else {
        "/"
    };

    Ok(Redirect::to(redirect_url))
}

/// Handler that logs the user out.
#[instrument(skip_all)]
pub(crate) async fn log_out(mut auth_session: AuthSession) -> Result<impl IntoResponse, HandlerError> {
    auth_session.logout().await?;

    Ok(Redirect::to(LOG_IN_URL))
}

/// Handler that signs up a new user.
#[instrument(skip_all)]
pub(crate) async fn sign_up(
    State(db): State<DynDB>,
    JobBoardId(job_board_id): JobBoardId,
    Form(mut new_user): Form<auth::NewUser>,
) -> Result<impl IntoResponse, HandlerError> {
    new_user.password = password_auth::generate_hash(&new_user.password);
    db.sign_up_user(&job_board_id, &new_user).await?;

    Ok(Redirect::to(LOG_IN_URL))
}
