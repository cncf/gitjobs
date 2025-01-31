//! Custom middleware for handlers.

use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use tracing::instrument;
use uuid::Uuid;

use crate::{auth::AuthSession, db::DynDB};

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
